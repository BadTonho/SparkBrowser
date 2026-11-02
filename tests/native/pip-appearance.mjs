import assert from 'node:assert/strict';
import {withFloatingPlayer} from './pip-player.mjs';

export async function checkMinimalPlayer(context) {
  const {script,until,wait,request,sessionId,screenshot,floatingWindow}=context;
  const native=await floatingWindow();assert.ok(native.topInset<=8,'floating client starts without a Windows title bar');
  await withFloatingPlayer(context,async()=>{
    const ui=await script('const p=document.querySelector("[data-testid=pip-play]").getBoundingClientRect(),h=document.querySelector(".header").getBoundingClientRect(),f=document.querySelector(".footer").getBoundingClientRect();return {width:innerWidth,height:innerHeight,centerX:p.left+p.width/2,centerY:p.top+p.height/2,header:h.top,footer:f.bottom,domain:document.querySelector("[data-testid=pip-origin]").textContent,svg:document.querySelectorAll("button svg").length,buttons:document.querySelectorAll("button").length};');
    assert.equal(ui.width,400);assert.equal(ui.height,225);assert.ok(Math.abs(ui.centerX-200)<1&&Math.abs(ui.centerY-112.5)<1);
    assert.equal(ui.header,0);assert.equal(ui.footer,225);assert.equal(ui.domain,'127.0.0.1');assert.equal(ui.svg,ui.buttons);
    await script('document.activeElement?.blur();document.dispatchEvent(new PointerEvent("pointermove"));');
    await screenshot('pip-minimal-controls');
    await until(()=>script('return document.querySelector("[data-testid=pip-chrome]").dataset.visible==="false"'),'controls hide without interaction',4000);
    await until(()=>script('return getComputedStyle(document.querySelector("[data-testid=pip-chrome]")).opacity==="0";'),'fade completes');
    await screenshot('pip-minimal-hidden');
    await script('document.dispatchEvent(new KeyboardEvent("keydown",{key:"Tab",bubbles:true}));document.querySelector("[data-testid=pip-play]").focus();');
    await wait(2200);assert.equal(await script('return document.querySelector("[data-testid=pip-chrome]").dataset.visible'),'true','keyboard focus keeps controls visible');
    await script('document.querySelector("[data-testid=pip-play]").click();document.activeElement?.blur();');
    await wait(2200);assert.equal(await script('return document.querySelector("[data-testid=pip-chrome]").dataset.visible'),'true','paused source keeps controls visible');
    await script('document.querySelector("[data-testid=pip-play]").click();');
    // CDP pointer events do not press the OS mouse button used by a window drag.
    const before=await floatingWindow(),after=await floatingWindow('Drag');
    assert.ok(before.left!==after.left||before.top!==after.top,'header drags the frameless window');
    const size=await request('GET',`/session/${sessionId()}/window/rect`);
    const resized=await floatingWindow('Resize');
    assert.ok(resized.width>after.width&&resized.height>after.height,'frameless borders retain native resizing');
    await request('POST',`/session/${sessionId()}/window/rect`,size);
    await script('document.querySelector("[data-testid=pip-minimize]").click();');
  });
  assert.equal((await floatingWindow()).minimized,true,'custom minimize button minimizes the native window');
  assert.equal((await floatingWindow('Restore')).minimized,false,'isolated floating window restores');
}
