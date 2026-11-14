// Select an internal page by tab identity; never depend on WebDriver handle order.
export function newTabDriver(context){
  const {invoke,script,until,request,sessionId}=context;
  const active=()=>invoke('get_tabs').then(p=>p.tabs.find(t=>t.is_active));
  const inside=async(action,id)=>{
    const target=id??(await active()).id;
    const previous=await request('GET',`/session/${sessionId()}/window`);
    try{
      await until(async()=>{
        const handles=await request('GET',`/session/${sessionId()}/window/handles`);
        for(const handle of handles){
          try{await request('POST',`/session/${sessionId()}/window`,{handle});if(await script('return window.__sparkNewTabId===arguments[0]',target))return true;}
          catch(error){if(!/no such window|web view not found/i.test(String(error)))throw error;}
        }
        return false;
      },'new tab WebView '+target);
      return await action();
    }finally{await request('POST',`/session/${sessionId()}/window`,{handle:previous});}
  };
  const ready=async()=>{
    await until(async()=>{const tab=await active();return tab.url==='spark://newtab'&&!tab.navigation.loading;},'internal navigation settled');
    return inside(()=>until(()=>script('return Boolean(document.querySelector("#search")?.placeholder&&document.querySelector("#recent")?.dataset.key)'),'local new tab ready'));
  };
  return {inside,ready,active};
}
