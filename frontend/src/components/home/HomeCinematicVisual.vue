<template>
  <div
    class="cinematic-stage"
    :class="[`scene-${scene}`, `direction-${direction}`, { 'reduced-motion': reducedMotion }]"
    :data-scene="scene"
    aria-hidden="true"
  >
    <div class="cinematic-shell">
      <div class="stage-aura stage-aura-primary" />
      <div class="stage-aura stage-aura-secondary" />

      <svg
        class="circuit-field"
        viewBox="0 0 520 520"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient
            :id="orbitGradientId"
            x1="84"
            y1="80"
            x2="448"
            y2="446"
            gradientUnits="userSpaceOnUse"
          >
            <stop
              stop-color="#F0BE97"
              stop-opacity="0.9"
            />
            <stop
              offset="0.48"
              stop-color="#D4A27F"
              stop-opacity="0.5"
            />
            <stop
              offset="1"
              stop-color="#B6D59C"
              stop-opacity="0.16"
            />
          </linearGradient>
          <radialGradient :id="pulseGradientId">
            <stop
              stop-color="#F0BE97"
              stop-opacity="0.5"
            />
            <stop
              offset="1"
              stop-color="#D4A27F"
              stop-opacity="0"
            />
          </radialGradient>
          <filter
            :id="glowFilterId"
            x="-200%"
            y="-200%"
            width="500%"
            height="500%"
          >
            <feGaussianBlur
              stdDeviation="5"
              result="blur"
            />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        <circle
          class="circuit-pulse"
          cx="260"
          cy="260"
          r="206"
          :fill="`url(#${pulseGradientId})`"
        />
        <g class="orbit orbit-outer">
          <ellipse
            cx="260"
            cy="260"
            rx="214"
            ry="138"
            :stroke="`url(#${orbitGradientId})`"
            stroke-width="1.2"
          />
          <circle
            cx="464"
            cy="220"
            r="4"
            fill="#F0BE97"
            :filter="`url(#${glowFilterId})`"
          />
        </g>
        <g class="orbit orbit-middle">
          <ellipse
            cx="260"
            cy="260"
            rx="176"
            ry="188"
            stroke="#D4A27F"
            stroke-opacity="0.24"
            stroke-dasharray="5 12"
          />
          <circle
            cx="260"
            cy="72"
            r="3.5"
            fill="#B6D59C"
            :filter="`url(#${glowFilterId})`"
          />
        </g>
        <g class="orbit orbit-inner">
          <ellipse
            cx="260"
            cy="260"
            rx="132"
            ry="96"
            stroke="#F0BE97"
            stroke-opacity="0.18"
          />
          <circle
            cx="382"
            cy="224"
            r="3"
            fill="#F0BE97"
            :filter="`url(#${glowFilterId})`"
          />
        </g>

        <g class="signal-paths">
          <path
            id="cinematic-signal-a"
            d="M66 292C138 132 360 102 458 246"
          />
          <path
            id="cinematic-signal-b"
            d="M82 360C206 464 394 418 450 274"
          />
          <circle
            class="signal-dot signal-dot-a"
            r="3.5"
            fill="#F0BE97"
            :filter="`url(#${glowFilterId})`"
          >
            <animateMotion
              path="M66 292C138 132 360 102 458 246"
              dur="5.2s"
              repeatCount="indefinite"
            />
          </circle>
          <circle
            class="signal-dot signal-dot-b"
            r="3"
            fill="#B6D59C"
            :filter="`url(#${glowFilterId})`"
          >
            <animateMotion
              path="M82 360C206 464 394 418 450 274"
              begin="-2.1s"
              dur="4.7s"
              repeatCount="indefinite"
            />
          </circle>
        </g>
      </svg>

      <div class="brand-core">
        <NifflerCinematicLogo :reduced-motion="reducedMotion" />
      </div>

      <div class="tool-satellites">
        <span class="satellite satellite-canvas">∞</span>
        <span class="satellite satellite-studio">✦</span>
        <span class="satellite-bridge" />
      </div>

      <div class="model-constellation">
        <span
          v-for="node in modelNodes"
          :key="node"
          class="model-node"
          :style="{ '--node-index': node }"
        />
      </div>

      <div class="faq-halo">
        <span class="faq-arc faq-arc-one" />
        <span class="faq-arc faq-arc-two" />
        <span class="faq-mark">?</span>
      </div>

      <div class="cta-convergence">
        <span class="convergence-line convergence-line-one" />
        <span class="convergence-line convergence-line-two" />
        <span class="convergence-line convergence-line-three" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import NifflerCinematicLogo from '@/components/home/NifflerCinematicLogo.vue'

export type HomeCinematicScene = 'hero' | 'tools' | 'models' | 'faq' | 'cta'
export type HomeScrollDirection = 'up' | 'down'

defineProps<{
  scene: HomeCinematicScene
  direction: HomeScrollDirection
  reducedMotion: boolean
}>()

const modelNodes = [0, 1, 2, 3, 4, 5]
const visualId = Math.random().toString(36).slice(2, 9)
const orbitGradientId = `home-orbit-${visualId}`
const pulseGradientId = `home-pulse-${visualId}`
const glowFilterId = `home-glow-${visualId}`
</script>

<style scoped>
.cinematic-stage {
  --stage-x: 18vw;
  --stage-y: -8vh;
  --stage-scale: 1.08;
  --stage-opacity: 0.2;
  --stage-rotation: -4deg;
  position: fixed;
  inset: 4rem 0 0;
  z-index: 0;
  pointer-events: none;
  overflow: hidden;
  opacity: var(--stage-opacity);
  transition:
    opacity 650ms ease-out,
    filter 650ms ease-out;
}

.cinematic-shell {
  position: absolute;
  left: 50%;
  top: 50%;
  width: min(42vw, 560px);
  aspect-ratio: 1;
  transform: translate3d(calc(-50% + var(--stage-x)), calc(-50% + var(--stage-y)), 0) scale(var(--stage-scale)) rotate(var(--stage-rotation));
  transform-origin: center;
  transition: transform 820ms cubic-bezier(0.16, 1, 0.3, 1);
  will-change: transform;
}

.direction-up .cinematic-shell {
  transition-timing-function: cubic-bezier(0.34, 1.38, 0.64, 1);
}

.scene-tools {
  --stage-x: -27vw;
  --stage-y: 1vh;
  --stage-scale: 0.94;
  --stage-opacity: 0.52;
  --stage-rotation: -7deg;
}

.scene-models {
  --stage-x: 28vw;
  --stage-y: 0;
  --stage-scale: 0.98;
  --stage-opacity: 0.46;
  --stage-rotation: 6deg;
}

.scene-faq {
  --stage-x: -29vw;
  --stage-y: 5vh;
  --stage-scale: 0.86;
  --stage-opacity: 0.16;
  --stage-rotation: -10deg;
}

.scene-cta {
  --stage-x: 0;
  --stage-y: 0;
  --stage-scale: 1.34;
  --stage-opacity: 0.65;
  --stage-rotation: 0deg;
  filter: saturate(1.15);
}

.stage-aura {
  position: absolute;
  border-radius: 9999px;
  filter: blur(42px);
}

.stage-aura-primary {
  inset: 13%;
  background: rgb(212 162 127 / 0.18);
  animation: aura-breathe 5.5s ease-in-out infinite;
}

.stage-aura-secondary {
  inset: 29% 8% 8% 36%;
  background: rgb(182 213 156 / 0.12);
  animation: aura-drift 8s ease-in-out infinite;
}

.circuit-field {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.circuit-pulse {
  transform-origin: center;
  animation: circuit-pulse 4.6s ease-in-out infinite;
}

.orbit {
  transform-box: fill-box;
  transform-origin: center;
}

.orbit-outer { animation: orbit-clockwise 22s linear infinite; }
.orbit-middle { animation: orbit-counter 28s linear infinite; }
.orbit-inner { animation: orbit-clockwise 16s linear infinite; }

.signal-paths path {
  fill: none;
  stroke: #d4a27f;
  stroke-width: 1;
  stroke-opacity: 0.12;
  stroke-dasharray: 4 12;
}

.brand-core {
  position: absolute;
  inset: 18%;
  display: grid;
  place-items: center;
  transform: scale(0.76);
  filter: drop-shadow(0 12px 34px rgb(212 162 127 / 0.18));
  transition: transform 760ms cubic-bezier(0.16, 1, 0.3, 1), filter 650ms ease;
}

.brand-core :deep(.niffler-cinematic-logo) {
  width: 100% !important;
  height: 100% !important;
}

.tool-satellites,
.model-constellation,
.faq-halo,
.cta-convergence {
  opacity: 0;
  transition: opacity 360ms ease, transform 700ms cubic-bezier(0.16, 1, 0.3, 1);
}

.tool-satellites {
  position: absolute;
  inset: 0;
  transform: scale(0.75) rotate(-12deg);
}

.scene-tools .tool-satellites {
  opacity: 1;
  transform: scale(1) rotate(0deg);
}

.satellite {
  position: absolute;
  display: grid;
  width: 58px;
  height: 58px;
  place-items: center;
  border: 1px solid rgb(212 162 127 / 0.45);
  border-radius: 9999px;
  background: rgb(250 250 247 / 0.76);
  color: #9a5a42;
  box-shadow: 0 0 34px rgb(212 162 127 / 0.14);
  font-family: ui-serif, Georgia, serif;
  font-size: 1.6rem;
  backdrop-filter: blur(8px);
  animation: satellite-float 4.8s ease-in-out infinite;
}

.satellite-canvas { left: 0; top: 43%; }
.satellite-studio { right: 0; top: 22%; animation-delay: -2.2s; }

.satellite-bridge {
  position: absolute;
  left: 10%;
  right: 10%;
  top: 48%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgb(212 162 127 / 0.55), transparent);
  transform: rotate(-13deg) scaleX(0.8);
  transform-origin: center;
  animation: bridge-signal 2.6s ease-in-out infinite;
}

.model-constellation {
  position: absolute;
  inset: 0;
  transform: scale(0.72) rotate(-18deg);
}

.scene-models .model-constellation {
  opacity: 1;
  transform: scale(1) rotate(0deg);
}

.model-node {
  --angle: calc(var(--node-index) * 60deg);
  position: absolute;
  left: calc(50% - 8px);
  top: calc(50% - 8px);
  width: 16px;
  height: 16px;
  border: 2px solid rgb(212 162 127 / 0.7);
  border-radius: 9999px;
  background: var(--background);
  box-shadow: 0 0 18px rgb(212 162 127 / 0.28);
  transform: rotate(var(--angle)) translateX(238px) rotate(calc(var(--angle) * -1));
  animation: model-node-pulse 3s ease-in-out infinite;
  animation-delay: calc(var(--node-index) * -0.42s);
}

.faq-halo {
  position: absolute;
  inset: 12%;
  transform: scale(0.78);
}

.scene-faq .faq-halo {
  opacity: 0.85;
  transform: scale(1);
}

.faq-arc {
  position: absolute;
  inset: 7%;
  border: 1px solid rgb(212 162 127 / 0.34);
  border-radius: 50%;
  clip-path: polygon(0 0, 54% 0, 54% 100%, 0 100%);
  animation: faq-orbit 9s linear infinite;
}

.faq-arc-two {
  inset: 20%;
  animation-direction: reverse;
  animation-duration: 7s;
}

.faq-mark {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: rgb(212 162 127 / 0.42);
  font-family: ui-serif, Georgia, serif;
  font-size: 7rem;
}

.cta-convergence {
  position: absolute;
  inset: 0;
  transform: scale(1.18);
}

.scene-cta .cta-convergence {
  opacity: 1;
  transform: scale(1);
}

.scene-cta .brand-core {
  transform: scale(0.88);
  filter: drop-shadow(0 0 42px rgb(240 190 151 / 0.42));
}

.convergence-line {
  position: absolute;
  left: 50%;
  top: 50%;
  width: 46%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgb(240 190 151 / 0.7));
  transform-origin: right center;
  animation: convergence 2.4s ease-in-out infinite;
}

.convergence-line-one { transform: translate(-100%, -50%) rotate(0deg); }
.convergence-line-two { transform: translate(-100%, -50%) rotate(120deg); animation-delay: -0.8s; }
.convergence-line-three { transform: translate(-100%, -50%) rotate(240deg); animation-delay: -1.6s; }

.reduced-motion,
.reduced-motion .cinematic-shell,
.reduced-motion .brand-core,
.reduced-motion .tool-satellites,
.reduced-motion .model-constellation,
.reduced-motion .faq-halo,
.reduced-motion .cta-convergence {
  transition: none;
}

.reduced-motion * {
  animation: none !important;
}

.reduced-motion .signal-dot { display: none; }
@keyframes orbit-clockwise { to { transform: rotate(360deg); } }
@keyframes orbit-counter { to { transform: rotate(-360deg); } }
@keyframes faq-orbit { to { transform: rotate(360deg); } }

@keyframes aura-breathe {
  0%, 100% { opacity: 0.56; transform: scale(0.92); }
  50% { opacity: 1; transform: scale(1.08); }
}

@keyframes aura-drift {
  0%, 100% { transform: translate3d(-14px, 8px, 0) scale(0.95); }
  50% { transform: translate3d(20px, -18px, 0) scale(1.1); }
}

@keyframes circuit-pulse {
  0%, 100% { opacity: 0.42; transform: scale(0.9); transform-origin: center; }
  50% { opacity: 0.9; transform: scale(1.06); transform-origin: center; }
}

@keyframes satellite-float {
  0%, 100% { transform: translateY(0) rotate(0deg); }
  50% { transform: translateY(-10px) rotate(5deg); }
}

@keyframes bridge-signal {
  0%, 100% { opacity: 0.24; transform: rotate(-13deg) scaleX(0.72); }
  50% { opacity: 1; transform: rotate(-13deg) scaleX(1); }
}

@keyframes model-node-pulse {
  0%, 100% { opacity: 0.45; box-shadow: 0 0 8px rgb(212 162 127 / 0.16); }
  50% { opacity: 1; box-shadow: 0 0 26px rgb(212 162 127 / 0.42); }
}

@keyframes convergence {
  0%, 100% { opacity: 0; width: 52%; }
  45% { opacity: 0.9; }
  75% { opacity: 0; width: 8%; }
}

@media (max-width: 1023px) {
  .cinematic-stage {
    position: absolute;
    inset: 0;
    opacity: 0.09;
  }

  .cinematic-shell {
    left: 70%;
    top: 16rem;
    width: min(92vw, 460px);
    transform: translate3d(-50%, -50%, 0) scale(0.78);
  }

  .tool-satellites,
  .model-constellation,
  .faq-halo,
  .cta-convergence { display: none; }
}

@media (prefers-reduced-motion: reduce) {
  .cinematic-stage,
  .cinematic-shell,
  .brand-core,
  .tool-satellites,
  .model-constellation,
  .faq-halo,
  .cta-convergence {
    transition: none;
  }

  .cinematic-stage * { animation: none !important; }
  .signal-dot { display: none; }
}
</style>
