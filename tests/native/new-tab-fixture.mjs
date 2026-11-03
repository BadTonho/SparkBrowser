import {deflateSync} from 'node:zlib';

// Generated one-pixel PNG; no network assets or browsing data are used.
export function testIcon() {
  const crc=bytes=>{
    let value=0xffffffff;
    for(const byte of bytes){value^=byte;for(let bit=0;bit<8;bit++)value=(value>>>1)^((value&1)?0xedb88320:0);}
    return (value^0xffffffff)>>>0;
  };
  const chunk=(type,data)=>{
    const name=Buffer.from(type),size=Buffer.alloc(4),sum=Buffer.alloc(4);
    size.writeUInt32BE(data.length);sum.writeUInt32BE(crc(Buffer.concat([name,data])));
    return Buffer.concat([size,name,data,sum]);
  };
  const header=Buffer.alloc(13);header.writeUInt32BE(1);header.writeUInt32BE(1,4);header[8]=8;header[9]=6;
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(Buffer.from([0,80,140,220,255]))),chunk('IEND',Buffer.alloc(0))]);
}

export function iconFixture() {
  const state={requests:[],pending:[],png:testIcon()};
  state.respond=(response,bad=false)=>{if(!response.destroyed)response.writeHead(200,{'Content-Type':'image/png','Cache-Control':'no-store'}).end(bad?Buffer.from('invalid icon'):state.png);};
  state.release=()=>{for(const response of state.pending.splice(0))state.respond(response);};
  state.handle=(url,response)=>{
    if(url.pathname!=='/new-tab-icon.png')return false;
    state.requests.push(url.search);
    if(url.searchParams.has('delay'))state.pending.push(response);else state.respond(response,url.searchParams.has('bad'));
    return true;
  };
  return state;
}
