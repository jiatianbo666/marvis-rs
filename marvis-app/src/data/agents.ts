export type AgentStatus = 'idle' | 'dispatching' | 'thinking' | 'working' | 'done' | 'error' | 'paused';
export type IdleVariant = 'sleeping' | 'coffee' | 'workout' | 'bathroom' | 'wandering';

export interface AgentDef {
  id: string;
  name: string;
  displayName: string;
  role: string;
  pet: string;
  color: string;
}

export interface AgentState extends AgentDef {
  status: AgentStatus;
  message: string;
  tokenUsed: number;
}

export const AGENTS: AgentDef[] = [
  { id: 'orchestrator',  name: 'Orchestrator',  displayName: '🎮 Cute Bear', role: 'PM / Scheduler',  pet: '🐻', color: '#8b5cf6' },
  { id: 'file_agent',    name: 'FileAgent',     displayName: '🦒 Giraffe',  role: 'File Scanner',    pet: '🦒', color: '#f59e0b' },
  { id: 'computer_agent',name: 'ComputerAgent', displayName: '🦙 Llama',    role: 'System Monitor',  pet: '🦙', color: '#3b82f6' },
  { id: 'app_agent',     name: 'AppAgent',      displayName: '🐰 Rabbit',   role: 'App Manager',     pet: '🐰', color: '#ec4899' },
  { id: 'browser_agent', name: 'BrowserAgent',  displayName: '🐶 Doggie',   role: 'Web Scraper',     pet: '🐶', color: '#06b6d4' },
  { id: 'search_agent',  name: 'SearchAgent',   displayName: '🐷 Pig',      role: 'Search Strategist',pet: '🐷', color: '#ef4444' },
];

export function getInitialAgents(): AgentState[] {
  return AGENTS.map(a => ({ ...a, status: 'idle' as AgentStatus, message: 'Ready', tokenUsed: 0 }));
}

/** 2x3 desk grid */
export const DESK_SPOTS: Record<string, { x: number; y: number }> = {
  orchestrator:   { x: 48, y: 20 },
  file_agent:     { x: 75, y: 20 },
  computer_agent: { x: 48, y: 50 },
  app_agent:      { x: 75, y: 50 },
  browser_agent:  { x: 48, y: 80 },
  search_agent:   { x: 75, y: 80 },
};

/** Public zones with coordinates */
export const PUBLIC_SPOTS: Record<string, { x: number; y: number }> = {
  coffee:   { x: 10, y: 19 },
  workout:  { x: 10, y: 49 },
  bathroom: { x: 10, y: 79 },
};

/** Zone layout for multi-agent positioning */
export const ZONE_LAYOUT: Record<string, { perRow: number; xGap: number; yGap: number }> = {
  coffee:   { perRow: 2, xGap: 14, yGap: 6 },
  workout:  { perRow: 2, xGap: 14, yGap: 6 },
  bathroom: { perRow: 2, xGap: 14, yGap: 6 },
};

export const MAX_ZONE_OCCUPANCY = 2;
