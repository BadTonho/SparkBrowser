import { spawn } from 'node:child_process';

/** Never let denied process termination hide the scenario's original failure. */
export function stopDriverProcess(driver, {
  platform = process.platform, spawnProcess = spawn,
  killGroup = process.kill, timeoutMs = 10_000,
} = {}) {
  if (!driver?.pid || driver.exitCode !== null || driver.signalCode !== null) return Promise.resolve();
  return new Promise((resolve, reject) => {
    let settled = false;
    const exited = () => finish();
    const timer = setTimeout(() => finish(new Error(`Timed out stopping isolated driver ${driver.pid}`)), timeoutMs);
    function finish(error) {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      driver.off('exit', exited);
      if (error) {
        // Report the surviving process explicitly without keeping Node hung on
        // its child handle or pipes when the OS refuses termination.
        driver.unref();
        driver.stderr?.destroy();
        driver.stdout?.destroy();
        reject(error);
      } else resolve();
    }
    driver.once('exit', exited);
    try {
      if (platform === 'win32') {
        const killer = spawnProcess('taskkill', ['/PID', String(driver.pid), '/T', '/F'], { windowsHide: true, stdio: 'ignore' });
        killer.unref?.();
        killer.once('error', error => finish(error));
        killer.once('close', code => {
          if (code !== 0) finish(new Error(`Cannot stop isolated driver ${driver.pid}: taskkill exited ${code}. Check process permissions.`));
        });
      } else killGroup(-driver.pid, 'SIGKILL');
    } catch (error) { finish(error); }
  });
}
