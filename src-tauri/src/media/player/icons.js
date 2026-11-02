(doc, name) => {
  const shapes = {
    play: [['path', {d:'M9 5.5 19 12 9 18.5Z',fill:'currentColor',stroke:'none'}]],
    pause: [['path', {d:'M8 6v12M16 6v12','stroke-width':'3'}]],
    back: [['path', {d:'M7 5V1M7 5h4M7 5a8 8 0 1 1-3 8'}],['text',{x:'12',y:'15','text-anchor':'middle','font-size':'7','font-family':'system-ui,sans-serif','font-weight':'600',fill:'currentColor',stroke:'none'},'10']],
    forward: [['path', {d:'M17 5V1M17 5h-4M17 5a8 8 0 1 0 3 8'}],['text',{x:'12',y:'15','text-anchor':'middle','font-size':'7','font-family':'system-ui,sans-serif','font-weight':'600',fill:'currentColor',stroke:'none'},'10']],
    speaker: [['path',{d:'M4 9h4l5-4v14l-5-4H4ZM16 8a6 6 0 0 1 0 8M19 5a10 10 0 0 1 0 14'}]],
    muted: [['path',{d:'M4 9h4l5-4v14l-5-4H4ZM17 9l5 6M22 9l-5 6'}]],
    return: [['path',{d:'M10 4H4v16h16v-6M13 4h7v7M20 4l-9 9'}]],
    minimize: [['path',{d:'M4 17h16'}]],
    close: [['path',{d:'m6 6 12 12M18 6 6 18'}]],
    site: [['rect',{x:'2',y:'5',width:'20',height:'14',rx:'4'}],['path',{d:'m10 9 5 3-5 3Z',fill:'currentColor',stroke:'none'}]],
  };
  const svg=doc.createElementNS('http://www.w3.org/2000/svg','svg');
  for(const [key,value] of Object.entries({viewBox:'0 0 24 24',fill:'none',stroke:'currentColor','stroke-width':'1.6','stroke-linecap':'round','stroke-linejoin':'round','aria-hidden':'true',focusable:'false'}))svg.setAttribute(key,value);
  for(const [tag,attributes,text] of shapes[name] ?? []) {
    const node=doc.createElementNS('http://www.w3.org/2000/svg',tag);
    for(const [key,value] of Object.entries(attributes))node.setAttribute(key,value);
    if(text)node.textContent=text;
    svg.append(node);
  }
  return svg;
}
