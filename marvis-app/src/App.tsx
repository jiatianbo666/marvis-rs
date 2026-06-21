import { useState, useCallback, useEffect, useRef } from "react";
import Office from "./components/Office";
import TokenBar from "./components/TokenBar";
import TaskLog from "./components/TaskLog";
import { getInitialAgents, AgentState, PendingConfirmation, AGENTS } from "./data/agents";
import "./App.css";

// 后端返回的状态格式
interface TaskStatus {
  running: boolean;
  stage: string;
  message: string;
  result: string;
  tokens: number;
  agents: Array<{ id: string; display: string; status: string; message: string }>;
}

function App() {
  const [agents, setAgents] = useState<Map<string, AgentState>>(() => {
    const initial = getInitialAgents();
    return new Map(initial.map(a => [a.id, a]));
  });
  const [taskLog, setTaskLog] = useState<string[]>([]);
  const [tokensUsed, setTokensUsed] = useState(0);
  const [taskInput, setTaskInput] = useState("");
  const [taskRunning, setTaskRunning] = useState(false);
  const [responseText, setResponseText] = useState("");
  const pollingRef = useRef<number | null>(null);

  // 从后端状态更新前端
  const applyTaskStatus = useCallback((s: TaskStatus) => {
    setTaskRunning(s.running);
    setTokensUsed(s.tokens);

    // 更新 agents
    setAgents(prev => {
      const next = new Map(prev);
      for (const a of s.agents) {
        const existing = next.get(a.id);
        if (existing) {
          next.set(a.id, { ...existing, status: a.status as any, message: a.message });
        }
      }
      return next;
    });

    // 结果
    if (s.result) setResponseText(s.result);
    if (s.stage === "done") {
      setTaskLog(prev => [...prev, `✅ Complete (${s.tokens} tokens)`].slice(-50));
    }
    if (s.stage === "error") {
      setTaskLog(prev => [...prev, `❌ ${s.message}`].slice(-50));
    }
  }, []);

  // 轮询后端
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
      } catch {
        pollingRef.current = null;
      }
    };
    poll();
  }, [applyTaskStatus]);

  useEffect(() => () => { if (pollingRef.current) clearTimeout(pollingRef.current); }, []);

  const handleStartTask = async () => {
    if (!taskInput.trim() || taskRunning) return;
    setResponseText("");
    setTaskLog(prev => [...prev, `💬 User: ${taskInput}`].slice(-50));

    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const status: TaskStatus = await invoke("run_task", { task: taskInput.trim() });
      applyTaskStatus(status);
      if (status.running) startPolling();
    } catch (_e) {
      setResponseText("❌ 无法连接到 Rust 后端。请确保通过 Tauri 应用运行，而非浏览器。");
      setTaskRunning(false);
    }
  };


  return (
    <div className="app-container">
      <div className="office-panel"><Office agents={agents} /></div>
      <div className="side-panel">
        <div className="app-title">🏢 Marvis Office</div>
        <div className="task-input-area">
          <textarea value={taskInput} onChange={e => setTaskInput(e.target.value)}
            placeholder="Describe a task for your AI office team..."
            rows={2} disabled={taskRunning}
            onKeyDown={e => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleStartTask(); }}} />
          <div className="button-row">
            <button className="btn-primary" onClick={handleStartTask} disabled={taskRunning || !taskInput.trim()}>
              {taskRunning ? '⏳ Running...' : '▶ Start'}
            </button>
            <button className="btn-secondary" onClick={() => setTaskInput("看看我的文件夹中有什么")} disabled={taskRunning}>🎲 Demo</button>
          </div>
        </div>
        <div className="status-bar">
          <span className="conn-dot connected" />
          <span>Rust Backend</span>
          <span className="stats-right">
            {Array.from(agents.values()).filter(a => a.status === 'working' || a.status === 'thinking').length} active
            &nbsp;|&nbsp;{Array.from(agents.values()).filter(a => a.status === 'done').length} done
          </span>
        </div>
        {responseText && (
          <div className="response-box">
            <div className="response-header">🤖 AI Response</div>
            <div className="response-content">{responseText.split('\n').map((line, i) => <div key={i}>{line || ' '}</div>)}</div>
          </div>
        )}
        <TokenBar tokensUsed={tokensUsed} />
        <TaskLog entries={taskLog} />
      </div>
    </div>
  );
}

export default App;
