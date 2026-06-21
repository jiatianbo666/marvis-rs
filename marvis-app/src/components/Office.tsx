import { AgentState, DESK_POSITIONS } from "../data/agents";
import AgentAvatar from "./AgentAvatar";

interface Props {
  agents: Map<string, AgentState>;
}

export default function Office({ agents }: Props) {
  const agentList = Array.from(agents.values());

  return (
    <div className="office-scene">
      {/* Background decorations */}
      <div className="office-bg">
        <div className="office-floor" />
        <div className="office-wall" />
        <div className="office-window left">🪟</div>
        <div className="office-window right">🪟</div>
        <div className="office-plant left">🪴</div>
        <div className="office-plant right">🌿</div>
        <div className="office-clock">🕐</div>
      </div>

      {/* Desks */}
      {Object.entries(DESK_POSITIONS).map(([agentId, pos]) => (
        <div
          key={agentId}
          className="office-desk"
          style={{ left: `${pos.x}%`, top: `${pos.y}%` }}
        >
          <div className="desk-icon">🖥️</div>
          <div className="desk-label">{agentId.replace('_', ' ')}</div>
        </div>
      ))}

      {/* Agents */}
      {agentList.map(agent => {
        const pos = DESK_POSITIONS[agent.id] || { x: 50, y: 50 };
        return (
          <div
            key={agent.id}
            style={{
              position: 'absolute',
              left: `${pos.x + 3}%`,
              top: `${pos.y - 5}%`,
            }}
          >
            <AgentAvatar agent={agent} size={52} />
          </div>
        );
      })}

      {/* Public zones */}
      <div className="zone coffee-zone" style={{ left: '80%', top: '15%' }}>
        <span className="zone-icon">☕</span>
        <span className="zone-label">Coffee</span>
      </div>
      <div className="zone gym-zone" style={{ left: '83%', top: '48%' }}>
        <span className="zone-icon">🏋️</span>
        <span className="zone-label">Gym</span>
      </div>
      <div className="zone rest-zone" style={{ left: '8%', top: '78%' }}>
        <span className="zone-icon">🚻</span>
        <span className="zone-label">Rest</span>
      </div>

      <style>{`
        .office-scene {
          width: 100%;
          height: 100%;
          position: relative;
          overflow: hidden;
        }

        .office-floor {
          position: absolute;
          bottom: 0;
          left: 0;
          right: 0;
          height: 92%;
          background: linear-gradient(180deg, #1a2a20 0%, #0f1a14 30%, #0a120e 100%);
        }

        .office-wall {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          height: 8%;
          background: linear-gradient(180deg, #2a3a30, #1a2a22);
          border-bottom: 1px solid #2a4a3a;
        }

        .office-window {
          position: absolute;
          top: 1%;
          font-size: 24px;
          opacity: 0.5;
        }
        .office-window.left { left: 25%; }
        .office-window.right { left: 55%; }

        .office-plant {
          position: absolute;
          bottom: 5%;
          font-size: 28px;
          opacity: 0.4;
        }
        .office-plant.left { left: 4%; }
        .office-plant.right { left: 75%; }

        .office-clock {
          position: absolute;
          top: 1.5%;
          right: 5%;
          font-size: 20px;
          opacity: 0.5;
        }

        .office-desk {
          position: absolute;
          display: flex;
          flex-direction: column;
          align-items: center;
          opacity: 0.5;
          pointer-events: none;
        }

        .desk-icon {
          font-size: 24px;
          filter: grayscale(0.3);
        }

        .desk-label {
          font-size: 9px;
          color: #4a6a5a;
          margin-top: 2px;
          text-transform: capitalize;
        }

        .zone {
          position: absolute;
          display: flex;
          flex-direction: column;
          align-items: center;
          border: 1px dashed #2a4a3a;
          border-radius: 12px;
          padding: 10px 14px;
          background: rgba(10, 30, 20, 0.6);
          opacity: 0.6;
        }

        .zone-icon {
          font-size: 22px;
        }

        .zone-label {
          font-size: 9px;
          color: #5a7a6a;
          margin-top: 4px;
        }
      `}</style>
    </div>
  );
}
