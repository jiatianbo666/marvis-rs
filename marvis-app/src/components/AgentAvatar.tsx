import { AgentState } from "../data/agents";

interface Props {
  agent: AgentState;
  size?: number;
}

export default function AgentAvatar({ agent, size = 56 }: Props) {
  const { status, petTheme, color, displayName, message } = agent;

  return (
    <div
      className={`agent-avatar state-${status}`}
      style={{
        width: size,
        height: size,
        fontSize: size * 0.55,
      }}
      title={`${displayName}: ${message}`}
    >
      <div
        className="avatar-pet"
        style={{ color }}
      >
        {petTheme}
      </div>

      {/* Status indicator */}
      <div className={`status-badge status-${status}`}>
        {status === 'working' ? '⚙' :
         status === 'done' ? '✓' :
         status === 'error' ? '✗' :
         status === 'paused' ? '⏸' :
         status === 'thinking' ? '💭' :
         status === 'dispatching' ? '📤' : ''}
      </div>

      {/* Agent name below */}
      <div className="agent-label" style={{ fontSize: 9 }}>
        {displayName.split(' ')[0]}
      </div>

      <style>{`
        .agent-avatar {
          position: absolute;
          display: flex;
          flex-direction: column;
          align-items: center;
          pointer-events: auto;
          cursor: pointer;
          z-index: 10;
          transition: left 0.8s cubic-bezier(0.4, 0, 0.2, 1),
                      top 0.8s cubic-bezier(0.4, 0, 0.2, 1);
        }

        .avatar-pet {
          width: 100%;
          height: 100%;
          border-radius: 50%;
          background: #132822;
          border: 2px solid currentColor;
          display: flex;
          align-items: center;
          justify-content: center;
          box-shadow: 0 2px 8px rgba(0,0,0,0.3);
          transition: transform 0.3s, box-shadow 0.3s;
        }

        .agent-avatar.state-idle .avatar-pet {
          animation: idleBounce 3s ease-in-out infinite;
        }
        .agent-avatar.state-working .avatar-pet,
        .agent-avatar.state-thinking .avatar-pet,
        .agent-avatar.state-dispatching .avatar-pet {
          animation: workingShake 0.6s ease-in-out infinite;
          box-shadow: 0 0 16px currentColor;
        }
        .agent-avatar.state-done .avatar-pet {
          animation: donePulse 1s ease-in-out infinite;
          box-shadow: 0 0 12px #22c55e;
        }
        .agent-avatar.state-error .avatar-pet {
          animation: errorWobble 0.4s ease-in-out infinite;
          box-shadow: 0 0 12px #ef4444;
        }
        .agent-avatar.state-paused .avatar-pet {
          animation: pausedFloat 2s ease-in-out infinite;
          box-shadow: 0 0 12px #f59e0b;
        }

        .status-badge {
          position: absolute;
          top: -4px;
          right: -4px;
          width: 18px;
          height: 18px;
          border-radius: 50%;
          background: #1a3028;
          border: 1px solid #3a5a48;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 10px;
        }
        .status-badge.status-working { border-color: #f59e0b; }
        .status-badge.status-done { border-color: #22c55e; background: #14532d; }
        .status-badge.status-error { border-color: #ef4444; background: #7f1d1d; }

        .agent-label {
          color: #8a9a8a;
          white-space: nowrap;
          margin-top: 2px;
          font-weight: 500;
        }

        @keyframes idleBounce {
          0%, 100% { transform: translateY(0); }
          50% { transform: translateY(-4px); }
        }
        @keyframes workingShake {
          0%, 100% { transform: translateX(0) rotate(0deg); }
          25% { transform: translateX(-3px) rotate(-2deg); }
          75% { transform: translateX(3px) rotate(2deg); }
        }
        @keyframes donePulse {
          0%, 100% { transform: scale(1); }
          50% { transform: scale(1.12); }
        }
        @keyframes errorWobble {
          0%, 100% { transform: rotate(0deg); }
          25% { transform: rotate(-8deg); }
          75% { transform: rotate(8deg); }
        }
        @keyframes pausedFloat {
          0%, 100% { transform: translateY(0); }
          50% { transform: translateY(-6px); }
        }
      `}</style>
    </div>
  );
}
