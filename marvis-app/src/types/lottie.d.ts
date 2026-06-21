declare module "*.svg" {
  const src: string;
  export default src;
}

declare module "lottie-web/build/player/lottie_svg.min.js" {
  interface AnimationItem {
    destroy(): void;
  }
  interface LoadParams {
    container: Element;
    renderer?: string;
    loop?: boolean;
    autoplay?: boolean;
    animationData?: object;
  }
  const lottie: { loadAnimation(params: LoadParams): AnimationItem };
  export default lottie;
}
