import React, { useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { ArrowLeft, ChevronDown, Clock3, Download, RotateCcw, Settings2, Timer, Upload, X } from 'lucide-react';
import { defaults, Settings, validateSettings } from './settings';
import './style.css';

const native = '__TAURI_INTERNALS__' in window;
function App() {
 const [open,setOpen]=useState(false); const [page,setPage]=useState<'home'|'settings'>('home');
 const [settings,setSettings]=useState<Settings>(defaults); const [now,setNow]=useState(Date.now());
 const [seconds,setSeconds]=useState(300); const [end,setEnd]=useState<number|null>(null); const [error,setError]=useState('');
 const input=useRef<HTMLInputElement>(null); const announced=useRef(false);
 useEffect(()=>{ const t=setInterval(()=>setNow(Date.now()),1000); return ()=>clearInterval(t); },[]);
 useEffect(()=>{ if(native) invoke<unknown>('load_settings').then(v=>setSettings(validateSettings(v))).catch(e=>setError(`Settings: ${String(e)}`)); },[]);
 useEffect(()=>{ if(!native) return; let unlisten: (()=>void)|undefined; listen('open-settings',()=>{setOpen(true);setPage('settings')}).then(fn=>{unlisten=fn}); return ()=>unlisten?.(); },[]);
 useEffect(()=>{ if(native) getCurrentWindow().setSize(new LogicalSize(open?420:260,open?520:52)).catch(e=>setError(String(e))); },[open]);
 useEffect(()=>{ if(end!==null && now>=end && !announced.current){announced.current=true; setEnd(null); setSeconds(0); setOpen(true); setPage('home');} },[now,end]);
 const save=async (s:Settings)=>{setSettings(s);if(native) try{await invoke('save_settings',{settings:s});setError('')}catch(e){setError(`Could not save settings: ${String(e)}`)}};
 const remaining=end===null?seconds:Math.max(0,Math.ceil((end-now)/1000));
 const time=new Date(now).toLocaleTimeString([], {hour:'2-digit',minute:'2-digit',...(settings.showSeconds?{second:'2-digit' as const}:{})});
 function exportSettings(){const url=URL.createObjectURL(new Blob([JSON.stringify(settings,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download='vr-island-settings.json';a.click();URL.revokeObjectURL(url)}
 async function importSettings(e:React.ChangeEvent<HTMLInputElement>){try{const file=e.target.files?.[0];if(file){if(file.size>16384)throw new Error('File is too large');await save(validateSettings(JSON.parse(await file.text())));}}catch(err){setError(`Import failed: ${String(err)}`)}e.target.value=''}
 return <div className={`shell ${settings.theme}`} style={{'--accent':settings.accent,'--speed':`${settings.animationMs}ms`} as React.CSSProperties}>
  {!open ? <button className="pill" onClick={()=>setOpen(true)} aria-label="Expand VR Island"><span className="brand-dot"/><span>VR ISLAND</span><span className="pill-time">{time}</span><ChevronDown size={15}/></button> : <div className="panel">
   <header><div className="wordmark"><span className="brand-dot"/> VR ISLAND <span className="beta">EARLY BUILD</span></div><div className="header-actions">{page==='home'&&<button className="icon" aria-label="Settings" onClick={()=>setPage('settings')}><Settings2 size={18}/></button>}<button className="icon" aria-label="Collapse" onClick={()=>{setOpen(false);setPage('home')}}><X size={18}/></button></div></header>
   {page==='home'?<main><div className="eyebrow">YOUR WINDOWS. YOUR ISLAND.</div><div className="hero"><div><div className="big-time">{time}</div><div className="date">{new Date(now).toLocaleDateString(undefined,{weekday:'long',month:'long',day:'numeric'})}</div></div><div className="clock-icon"><Clock3 size={25}/></div></div>
    <div className="section-heading"><span>YOUR SPACE</span><span>01 / 01</span></div><div className="card"><div className="card-top"><div className="tile-icon"><Timer size={19}/></div><div><strong>Focus timer</strong><p>A little time, just for you.</p></div><span className="local">LOCAL</span></div><div className="timer-row"><div className="timer-value">{String(Math.floor(remaining/60)).padStart(2,'0')}:{String(remaining%60).padStart(2,'0')}</div><div className="timer-controls"><button onClick={()=>{setEnd(null);setSeconds(300);announced.current=false}} aria-label="Reset timer"><RotateCcw size={16}/></button><button className="primary" onClick={()=>{if(end!==null){setSeconds(remaining);setEnd(null)}else if(seconds>0){announced.current=false;setEnd(Date.now()+seconds*1000)}}}>{end!==null?'Pause':'Start'}</button></div></div><div className="presets">{[1,5,15,25].map(m=><button key={m} onClick={()=>{setEnd(null);setSeconds(m*60);announced.current=false}}>{m} min</button>)}</div>{remaining===0&&<div className="notice" role="status">Timer complete</div>}</div>
    <div className="note"><span className="note-dot"/> Windows integrations are not available in this early build.</div><footer>MADE BY VR DEVELOPMENTS <span>✦</span> BUILT FOR WINDOWS</footer></main> : <main className="settings"><button className="back" onClick={()=>setPage('home')}><ArrowLeft size={16}/> Back to island</button><h1>Settings</h1><p className="subtitle">Make the island yours.</p><div className="section-heading"><span>APPEARANCE</span></div><label>Theme<select value={settings.theme} onChange={e=>save({...settings,theme:e.target.value as Settings['theme']})}><option value="dark">Dark</option><option value="light">Light</option><option value="black">Black</option></select></label><label>Accent color<input type="color" value={settings.accent} onChange={e=>save({...settings,accent:e.target.value})}/></label><label>Show seconds<input type="checkbox" checked={settings.showSeconds} onChange={e=>save({...settings,showSeconds:e.target.checked})}/></label><label>Animation <span>{settings.animationMs} ms <input aria-label="Animation duration" type="range" min="0" max="1000" step="20" value={settings.animationMs} onChange={e=>save({...settings,animationMs:Number(e.target.value)})}/></span></label><div className="section-heading"><span>CONFIGURATION</span></div><div className="actions"><button onClick={exportSettings}><Download size={15}/> Export</button><button onClick={()=>input.current?.click()}><Upload size={15}/> Import</button><button onClick={()=>save(defaults)}><RotateCcw size={15}/> Defaults</button><input ref={input} hidden type="file" accept="application/json,.json" onChange={importSettings}/></div><p className="privacy">Settings stay on this device. No accounts or telemetry.</p>{error&&<p className="error" role="alert">{error}</p>}</main>}
  </div>}
 </div>;
}
createRoot(document.getElementById('root')!).render(<App/>);
