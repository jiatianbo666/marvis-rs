export type AgentStatus = 'idle' | 'dispatching' | 'thinking' | 'working' | 'done' | 'error' | 'paused';

export interface AgentDef {
  id: string;
  name: string;
  displayName: string;
  role: string;
  petTheme: string;   // emoji or color for the avatar
  color: string;       // accent color for the agent
}

export interface AgentState extends AgentDef {
  status: AgentStatus;
  message: string;
  tokenUsed: number;
}

export interface PendingConfirmation {
  confirmationId: string;
  agentName: string;
  agentDisplayName: string;
  action: string;
  detail: Record<string, unknown>;
}

export const StatusLabel: Record<AgentStatus, string> = {
  idle: 'Idle',
  dispatching: 'Dispatching',
  thinking: 'Thinking',
  working: 'Working',
  done: 'Done',
  error: 'Error',
  paused: 'Paused',
};

export const AGENTS: AgentDef[] = [
  { id: 'orchestrator', name: 'Orchestrator', displayName: '🎯 Panda PM', role: 'Task Planner & Coordinator', petTheme: '🐼', color: '#8b5cf6' },
  { id: 'file_agent', name: 'FileAgent', displayName: '🦊 Fox File Ops', role: 'File Scanner & Analyzer', petTheme: '🦊', color: '#f59e0b' },
  { id: 'computer_agent', name: 'ComputerAgent', displayName: '🐴 Horse SysInfo', role: 'System & Process Monitor', petTheme: '🐴', color: '#3b82f6' },
  { id: 'app_agent', name: 'AppAgent', displayName: '🐰 Rabbit Apps', role: 'App Manager & Launcher', petTheme: '🐰', color: '#ec4899' },
  { id: 'browser_agent', name: 'BrowserAgent', displayName: '🐶 Dog Browser', role: 'Web Scraper & Fetcher', petTheme: '🐶', color: '#06b6d4' },
  { id: 'search_agent', name: 'SearchAgent', displayName: '🐷 Pig Search', role: 'Search Strategist', petTheme: '🐷', color: '#ef4444' },
];

export function getInitialAgents(): AgentState[] {
  return AGENTS.map(a => ({
    ...a,
    status: 'idle' as AgentStatus,
    message: 'Ready',
    tokenUsed: 0,
  }));
}

/** Office desk positions as percentages */
export const DESK_POSITIONS: Record<string, { x: number; y: number }> = {
  orchestrator:  { x: 15, y: 20 },
  file_agent:    { x: 38, y: 20 },
  computer_agent:{ x: 61, y: 20 },
  app_agent:     { x: 15, y: 55 },
  browser_agent: { x: 38, y: 55 },
  search_agent:  { x: 61, y: 55 },
};

/** Public zones where idle agents wander */
export const PUBLIC_ZONES = [
  { id: 'coffee', label: '☕', x: 80, y: 20 },
  { id: 'gym', label: '🏋️', x: 85, y: 55 },
  { id: 'restroom', label: '🚻', x: 10, y: 85 },
];
