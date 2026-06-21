import { useEffect, useRef } from "react";
import computerAnim from "../assets/lottie/computer.json";

export default function DeskComputer() {
  const ref = useRef<HTMLDivElement>(null);
  const animRef = useRef<any>(null);

  useEffect(() => {
    if (!ref.current) return;
    let cancelled = false;
    import("lottie-web/build/player/lottie_svg.min.js").then((m: any) => {
      if (cancelled || !ref.current) return;
      const lot = m.default || m;
      animRef.current?.destroy();
      animRef.current = lot.loadAnimation({
        container: ref.current,
        renderer: "svg",
        loop: true,
        autoplay: true,
        animationData: computerAnim,
        rendererSettings: { preserveAspectRatio: "xMidYMid meet" },
      });
    }).catch(() => {});
    return () => { cancelled = true; animRef.current?.destroy(); };
  }, []);

  return <div ref={ref} style={{ width: 80, height: 60 }} />;
}
