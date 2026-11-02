import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { readFileSync } from 'node:fs';

export async function checkKeyboard(context) {
  if (process.platform !== 'win32') return;
  const {invoke,until,fixture,wait,script,click}=context;
  const build=JSON.parse(readFileSync('.cache/native-target/test-build.json','utf8'));
  assert.equal(build.identifier,'org.veloce.browser.native-tests');
  const executable=resolve(build.executable);
  assert.ok(executable.startsWith(resolve('.cache/native-target')+'\\'));
  const key=(code,up=false)=>new Promise((resolveKey,reject)=>{
    const child=spawn('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',resolve('tests/native/windows-input.ps1'),'-Executable',executable,'-Key',String(code),...(up?['-Up']:[])],{windowsHide:true,stdio:['ignore','ignore','pipe']});
    let error='';child.stderr.on('data',data=>error+=data);child.on('error',reject);child.on('exit',code=>code===0?resolveKey():reject(new Error(error||'Native key helper failed')));
  });
  await invoke('set_resident_limit',{limit:20});
  const ids=[];
  for(let i=0;i<5;i++){
    const page='keyboard-'+i;
    const packet=await invoke('create_tab',{url:fixture.origin+'/page/'+page});ids.push(packet.tabs.find(t=>t.is_active).id);
    await until(()=>fixture.reports.has(page),'keyboard fixture loaded');
  }
  const initial=(await invoke('get_tabs')).tabs.length;
  let failure;
  try {
    await key(0x11);
    for(let i=0;i<3;i++){
      await key(0x57);
      await until(async()=> (await invoke('get_tabs')).tabs.length===initial-i-1,`Ctrl held: close page ${i+1}`);
      await until(()=>fixture.reports.get('keyboard-'+(3-i))?.focused,'next native page has keyboard focus');
      if(i===0){
        await key(0x57); await wait(150);
        assert.equal((await invoke('get_tabs')).tabs.length,initial-1,'holding W must not repeat closing on the newly focused page');
      }
      await key(0x57,true);
    }
    assert.ok((await invoke('get_tabs')).tabs.some(t=>t.id===ids[0]));
    await key(0x10);
    for(let i=0;i<3;i++){
      await key(0x54);
      await until(async()=> (await invoke('get_tabs')).tabs.length===initial-2+i,`Ctrl+Shift held: reopen page ${i+1}`);
      await key(0x54,true);
    }
    await key(0x10,true);
    await click('.omnibox-input');
    await until(()=>script('return document.hasFocus() && document.activeElement?.matches(".omnibox-input")'),'shell keyboard focus');
    await key(0x57);
    await until(async()=> (await invoke('get_tabs')).tabs.length===initial-1,'close from shell with Ctrl still held');
    await key(0x57,true);
  } catch (error) {
    failure = error; throw error;
  } finally {
    const released = await Promise.allSettled([key(0x57,true),key(0x54,true),key(0x10,true),key(0x11,true)]);
    const errors = released.filter(result => result.status === 'rejected').map(result => String(result.reason));
    if (errors.length) {
      if (failure) console.error('Additional keyboard cleanup errors: '+errors.join('; '));
      else throw new Error('Could not release injected test keys: '+errors.join('; '));
    }
  }
}
