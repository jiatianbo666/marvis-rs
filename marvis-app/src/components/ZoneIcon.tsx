import { useEffect, useRef } from "react";
import coffeeAnim from "../assets/lottie/coffee.json";
import treadmillAnim from "../assets/lottie/treadmill.json";
import toiletSvg from "../assets/lottie/toilet.svg";

interface Props { zone: "coffee" | "gym" | "rest"; size?: number }

export default function ZoneIcon({ zone, size = 48 }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const animRef = useRef<any>(null);

  useEffect(() => {
    if (!ref.current) return;
    let cancelled = false;

    // Rest zone uses SVG image
    if (zone === "rest") {
      ref.current.innerHTML = `<img src="${toiletSvg}" style="width:100%;height:100%" />`;
      return () => { cancelled = true; if (ref.current) ref.current.innerHTML = ""; };
    }

    // Coffee and gym use Lottie
    const data = zone === "coffee" ? coffeeAnim : treadmillAnim;
    import("lottie-web/build/player/lottie_svg.min.js").then((m: any) => {
      if (cancelled || !ref.current) return;
      const lot = m.default || m;
      animRef.current?.destroy();
      animRef.current = lot.loadAnimation({
        container: ref.current,
        renderer: "svg",
        loop: true,
        autoplay: true,
        animationData: data,
        rendererSettings: { preserveAspectRatio: "xMidYMid meet" },
      });
    }).catch(() => {});
    return () => { cancelled = true; animRef.current?.destroy(); };
  }, [zone]);

  return <div ref={ref} style={{ width: size, height: size }} />;
}
