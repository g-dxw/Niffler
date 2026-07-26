<template>
  <div
    class="niffler-cinematic-logo"
    :class="{ 'reduced-motion': reducedMotion }"
  >
    <svg
      :viewBox="NIFFLER_LOGO_VIEWBOX"
      class="h-full w-full overflow-visible"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <defs>
        <linearGradient
          :id="fillGradientId"
          x1="12"
          y1="4"
          x2="112"
          y2="101"
          gradientUnits="userSpaceOnUse"
        >
          <stop stop-color="#F0BE97" />
          <stop
            offset="0.5"
            stop-color="#D4A27F"
          />
          <stop
            offset="1"
            stop-color="#B97A5C"
          />
        </linearGradient>
        <linearGradient
          :id="strokeGradientId"
          x1="18"
          y1="6"
          x2="111"
          y2="98"
          gradientUnits="userSpaceOnUse"
        >
          <stop stop-color="#F7D7BD" />
          <stop
            offset="0.46"
            stop-color="#D4A27F"
          />
          <stop
            offset="1"
            stop-color="#B6D59C"
          />
        </linearGradient>
        <filter
          :id="glowFilterId"
          x="-35%"
          y="-35%"
          width="170%"
          height="170%"
        >
          <feGaussianBlur
            stdDeviation="1.8"
            result="blur"
          />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>

      <path
        class="mark-aura"
        :d="NIFFLER_LOGO_PATH"
        :fill="`url(#${fillGradientId})`"
      />
      <path
        class="mark-fill"
        :d="NIFFLER_LOGO_PATH"
        :fill="`url(#${fillGradientId})`"
        fill-rule="evenodd"
      />
      <path
        class="mark-outline mark-outline-ghost"
        :d="NIFFLER_LOGO_PATH"
        fill="none"
        :stroke="`url(#${strokeGradientId})`"
        stroke-width="1.15"
        stroke-linecap="round"
        stroke-linejoin="round"
        vector-effect="non-scaling-stroke"
      />
      <path
        class="mark-outline mark-outline-draw"
        :d="NIFFLER_LOGO_PATH"
        fill="none"
        :stroke="`url(#${strokeGradientId})`"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
        pathLength="1"
        vector-effect="non-scaling-stroke"
        :filter="`url(#${glowFilterId})`"
      />
      <path
        class="mark-signal"
        :d="NIFFLER_LOGO_PATH"
        fill="none"
        stroke="#F7E7D9"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        pathLength="1"
        vector-effect="non-scaling-stroke"
        :filter="`url(#${glowFilterId})`"
      />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { NIFFLER_LOGO_PATH, NIFFLER_LOGO_VIEWBOX } from '@/constants/logoPaths'

defineProps<{
  reducedMotion: boolean
}>()

const visualId = Math.random().toString(36).slice(2, 9)
const fillGradientId = `niffler-fill-${visualId}`
const strokeGradientId = `niffler-stroke-${visualId}`
const glowFilterId = `niffler-glow-${visualId}`
</script>

<style scoped>
.niffler-cinematic-logo {
  width: 100%;
  height: 100%;
}

.mark-aura {
  opacity: 0.14;
  filter: blur(8px);
  transform-box: fill-box;
  transform-origin: center;
  animation: mark-aura 5.8s ease-in-out infinite;
}

.mark-fill {
  opacity: 0.22;
  animation: mark-fill 5.8s ease-in-out infinite;
}

.mark-outline-ghost { opacity: 0.26; }

.mark-outline-draw {
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  animation: mark-draw 5.8s cubic-bezier(0.16, 1, 0.3, 1) infinite;
}

.mark-signal {
  opacity: 0;
  stroke-dasharray: 0.035 0.965;
  stroke-dashoffset: 1;
  animation: mark-signal 5.8s linear infinite;
}

@keyframes mark-draw {
  0%, 8% { opacity: 0; stroke-dashoffset: 1; }
  38%, 72% { opacity: 0.9; stroke-dashoffset: 0; }
  92%, 100% { opacity: 0; stroke-dashoffset: -1; }
}

@keyframes mark-fill {
  0%, 18%, 100% { opacity: 0.08; }
  46%, 72% { opacity: 0.38; }
}

@keyframes mark-aura {
  0%, 100% { opacity: 0.08; transform: scale(0.94); }
  52% { opacity: 0.22; transform: scale(1.04); }
}

@keyframes mark-signal {
  0%, 28% { opacity: 0; stroke-dashoffset: 1; }
  40%, 72% { opacity: 0.9; }
  86%, 100% { opacity: 0; stroke-dashoffset: -1; }
}

.reduced-motion .mark-aura,
.reduced-motion .mark-fill,
.reduced-motion .mark-outline-draw,
.reduced-motion .mark-signal {
  animation: none;
}

.reduced-motion .mark-aura { opacity: 0.12; }
.reduced-motion .mark-fill { opacity: 0.3; }
.reduced-motion .mark-outline-draw { opacity: 0.75; stroke-dashoffset: 0; }
.reduced-motion .mark-signal { display: none; }
</style>
