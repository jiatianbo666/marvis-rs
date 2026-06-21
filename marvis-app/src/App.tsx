import { useState, useCallback, useEffect, useRef, useMemo } from "react";
import Office from "./components/Office";
import TaskLog from "./components/TaskLog";
import { getInitialAgents, AgentState, IdleVariant, PUBLIC_SPOTS, MAX_ZONE_OCCUPANCY, AGENTS } from "./data/agents";
import "./App.css";

interface TaskStatus {
  running: boolean; stage: string; message: string;
  result: string; tokens: number;
  agents: Array<{ id: string; display: string; status: string; message: string }>;
}

const ALL_VARIANTS: IdleVariant[] = ['sleeping', 'coffee', 'workout', 'bathroom', 'wandering'];

// ====== Idle variant assignment (from marvis-office) ======
function assignIdleVariants(
  agents: Map<string, AgentState>,
  prev: Map<string, IdleVariant>,
  randomize: boolean
): Map<string, IdleVariant> {
  const next = new Map<string, IdleVariant>();
  let changed = false;

  agents.forEach((agent, id) => {
    if (agent.status === 'idle') {
      let variant: IdleVariant;

      if (randomize || !prev.has(id)) {
        variant = ALL_VARIANTS[Math.floor(Math.random() * ALL_VARIANTS.length)];
      } else {
        variant = prev.get(id) || 'sleeping';
      }

      // Check zone capacity
      if (variant === 'coffee' || variant === 'workout' || variant === 'bathroom') {
        let count = 0;
        next.forEach((v) => { if (v === variant) count++; });
        if (count >= MAX_ZONE_OCCUPANCY) {
          variant = Math.random() > 0.5 ? 'sleeping' : 'wandering';
        }
      }

      next.set(id, variant);
      if (prev.get(id) !== variant) changed = true;
    } else {
      next.set(id, prev.get(id) || 'sleeping');
    }
  });

  return changed ? next : prev;
}

function App() {
  const [agents, setAgents] = useState<Map<string, AgentState>>(() => {
    const initial = getInitialAgents();
    return new Map(initial.map(a => [a.id, a]));
  });
  const [idleVariants, setIdleVariants] = useState<Map<string, IdleVariant>>(() => {
    const m = new Map<string, IdleVariant>();
    getInitialAgents().forEach(a => m.set(a.id, 'sleeping'));
    return m;
  });
  const [taskLog, setTaskLog] = useState<string[]>([]);
  const [tokensUsed, setTokensUsed] = useState(0);
  const [taskInput, setTaskInput] = useState("");
  const [taskRunning, setTaskRunning] = useState(false);
  const [responseText, setResponseText] = useState("");
  const pollingRef = useRef<number | null>(null);

  // ====== 18-second idle rotation ======
  useEffect(() => {
    const timer = window.setInterval(() => {
      setAgents(prev => {
        setIdleVariants(prevVariants => assignIdleVariants(prev, prevVariants, true));
        return prev;
      });
    }, 18000);
    return () => window.clearInterval(timer);
  }, []);

  // ====== Apply backend status ======
  const applyTaskStatus = useCallback((s: TaskStatus) => {
    setTaskRunning(s.running);
    setTokensUsed(s.tokens);
    if (s.result) setResponseText(s.result);

    setAgents(prev => {
      const next = new Map(prev);
      const newLogs: string[] = [];

      for (const a of s.agents) {
        const existing = next.get(a.id);
        if (existing) {
          // Detect status change → log it
          if (a.status !== existing.status && a.status !== "idle") {
            const emoji = AGENTS.find(ag => ag.id === a.id)?.pet || "";
            const name = AGENTS.find(ag => ag.id === a.id)?.displayName || a.id;
            if (a.status === "thinking") newLogs.push(`💭 ${emoji} ${name}: ${a.message || "thinking..."}`);
            else if (a.status === "working") newLogs.push(`⚙️ ${emoji} ${name}: ${a.message || "working..."}`);
            else if (a.status === "dispatching") newLogs.push(`📤 ${emoji} ${name}: ${a.message || "dispatching..."}`);
            else if (a.status === "done") newLogs.push(`✅ ${emoji} ${name}: ${a.message || "done!"}`);
            else if (a.status === "error") newLogs.push(`❌ ${emoji} ${name}: ${a.message || "error!"}`);
          }
          next.set(a.id, { ...existing, status: a.status as any, message: a.message });
        }
      }

      if (newLogs.length > 0) {
        setTaskLog(prev => [...prev, ...newLogs].slice(-80));
      }

      setIdleVariants(prevV => assignIdleVariants(next, prevV, false));
      return next;
    });

    if (s.stage === "done" || s.stage === "error") {
      setTaskLog(prev => [...prev, `${s.stage === "done" ? "🏁" : "💥"} ${s.message} (${s.tokens} tokens)`].slice(-80));
    }
  }, []);

  // ====== Polling ======
  const startPolling = useCallback(() => {
    if (pollingRef.current) return;
    const poll = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const status: TaskStatus = await invoke("get_task_status");
        applyTaskStatus(status);
        if (status.running) {
          pollingRef.current = window.setTimeout(poll, 500);
        } else {
          pollingRef.current = null;
        }
      } catch { pollingRef.current = null; }
    };
    poll();
  }, [applyTaskStatus]);

  useEffect(() => () => { if (pollingRef.current) clearTimeout(pollingRef.current); }, []);

  // ====== Start task ======
  const handleStartTask = async () => {
    if (!taskInput.trim() || taskRunning) return;
    setResponseText("");
    setTaskLog(prev => [...prev, `💬 ${taskInput}`].slice(-50));

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const status: TaskStatus = await invoke("run_task", { task: taskInput.trim() });
      applyTaskStatus(status);
      if (status.running) startPolling();
    } catch (_e) {
      setResponseText("❌ 无法连接到 Rust 后端。请确保通过 Tauri 应用运行。");
    }
  };

  // ====== Stats ======
  const activeCount = useMemo(() =>
    Array.from(agents.values()).filter(a => a.status === 'working' || a.status === 'thinking' || a.status === 'dispatching').length,
  [agents]);
  const doneCount = useMemo(() =>
    Array.from(agents.values()).filter(a => a.status === 'done').length,
  [agents]);

  return (
    <div className="app-layout">
      <div className="office-panel">
        <Office agents={agents} idleVariants={idleVariants} />
      </div>

      <aside className="side-panel">
        <div className="app-title">🏢 Marvis Office</div>

        <textarea className="task-input" value={taskInput}
          onChange={e => setTaskInput(e.target.value)}
          placeholder="Describe a task for your AI office team..."
          rows={2} disabled={taskRunning}
          onKeyDown={e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleStartTask(); }}} />

        <div className="button-row">
          <button className="btn-primary" onClick={handleStartTask} disabled={taskRunning || !taskInput.trim()}>
            {taskRunning ? '⏳ Running...' : '▶ Start'}
          </button>
          <button className="btn-secondary" onClick={() => setTaskInput("查看系统信息并列出文件夹")} disabled={taskRunning}>🎲 Demo</button>
        </div>

        <div className="status-bar">
          <span className="conn-dot connected" />
          <span>Rust Backend</span>
          <span className="stats-right">{activeCount} active | {doneCount} done</span>
        </div>

        {responseText && (
          <div className="response-box">
            <div className="response-header">🤖 AI Response</div>
            <div className="response-content">{responseText.split('\n').map((l, i) => <div key={i}>{l || ' '}</div>)}</div>
          </div>
        )}

        <TaskLog entries={taskLog} />
      </aside>
    </div>
  );
}

export default App;
