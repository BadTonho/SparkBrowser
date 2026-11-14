import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

// Explicit network check; ad delivery is not deterministic and is never required.
export async function checkYoutubeAutoSkip({ invoke, withWebview, script, wait, directory }) {
  await invoke('set_auto_skip_ads', { enabled: true });
  await invoke('navigate_to', { input: 'https://www.youtube.com/watch?v=WKc0w7t6mzE' });
  const observations = [];
  await withWebview('/watch', async () => {
    await script(`window.fixtureSkipClicks=0;document.addEventListener('click',event=>{
      if(event.target.closest?.('#movie_player button.ytp-ad-skip-button,#movie_player button.ytp-skip-ad-button,#movie_player button.ytp-ad-skip-button-modern'))window.fixtureSkipClicks++;
    },true);document.querySelector('video')?.play().catch(()=>{});`);
    for (let i = 0; i < 10; i++) {
      await wait(2000);
      observations.push(await script(`const p=document.querySelector('#movie_player');return {
        player:!!p,ad:!!p?.classList.contains('ad-showing'),
        skipButtons:p?.querySelectorAll('button.ytp-ad-skip-button,button.ytp-skip-ad-button,button.ytp-ad-skip-button-modern').length??0,
        clicks:window.fixtureSkipClicks??0,controller:typeof window.__sparkSetAutoSkip==='function'};`));
    }
  });
  const outcome = observations.some(o => o.clicks > 0) ? 'skip-observed' : 'no-skippable-ad-observed';
  writeFileSync(resolve(directory, 'artifacts/auto-skip-youtube.json'), JSON.stringify({ outcome, observations }, null, 2));
  console.log('YouTube auto skip:', outcome);
}
