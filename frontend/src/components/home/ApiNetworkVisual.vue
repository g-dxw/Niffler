<template>
  <div class="api-network-visual">
    <svg
      class="h-full w-full"
      viewBox="0 0 560 380"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      role="group"
      :aria-labelledby="`${titleId} ${descriptionId}`"
    >
      <title :id="titleId">{{ accessibleTitle }}</title>
      <desc :id="descriptionId">{{ accessibleDescription }}</desc>

      <defs>
        <linearGradient
          :id="lineGradientId"
          x1="280"
          y1="20"
          x2="280"
          y2="370"
          gradientUnits="userSpaceOnUse"
        >
          <stop
            stop-color="#F0BE97"
            stop-opacity="0.96"
          />
          <stop
            offset="0.52"
            stop-color="#D4A27F"
            stop-opacity="0.72"
          />
          <stop
            offset="1"
            stop-color="#D4A27F"
            stop-opacity="0.2"
          />
        </linearGradient>
        <radialGradient :id="auraGradientId">
          <stop
            stop-color="#D4A27F"
            stop-opacity="0.32"
          />
          <stop
            offset="1"
            stop-color="#D4A27F"
            stop-opacity="0"
          />
        </radialGradient>
        <filter
          :id="glowFilterId"
          x="-80%"
          y="-80%"
          width="260%"
          height="260%"
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
        <filter
          :id="softGlowFilterId"
          x="-50%"
          y="-70%"
          width="200%"
          height="240%"
        >
          <feGaussianBlur stdDeviation="15" />
        </filter>
        <marker
          :id="arrowMarkerId"
          viewBox="0 0 8 8"
          refX="7"
          refY="4"
          markerWidth="5"
          markerHeight="5"
          orient="auto"
        >
          <path
            d="M0 0L8 4L0 8Z"
            fill="#F0BE97"
            fill-opacity="0.48"
          />
        </marker>
      </defs>

      <ellipse
        class="network-aura"
        cx="280"
        cy="208"
        rx="142"
        ry="82"
        :fill="`url(#${auraGradientId})`"
      />

      <g class="orbit orbit-outer">
        <ellipse
          cx="280"
          cy="208"
          rx="126"
          ry="58"
          stroke="#D4A27F"
          stroke-opacity="0.18"
        />
        <circle
          cx="397"
          cy="187"
          r="3.5"
          fill="#F0BE97"
          :filter="`url(#${glowFilterId})`"
        />
      </g>
      <g class="orbit orbit-inner">
        <ellipse
          cx="280"
          cy="208"
          rx="102"
          ry="74"
          stroke="#D4A27F"
          stroke-opacity="0.12"
          stroke-dasharray="3 9"
        />
        <circle
          cx="280"
          cy="134"
          r="3"
          fill="#B6D59C"
          :filter="`url(#${glowFilterId})`"
        />
      </g>

      <g class="connection-field">
        <g
          v-for="connection in upstreamConnections"
          :key="`upstream-${connection.id}`"
          class="connection"
          :class="connectionClasses('upstream', connection.id)"
          data-flow-side="upstream"
          :data-node-id="connection.id"
        >
          <path
            :d="connection.path"
            class="connection-base"
            :marker-end="`url(#${arrowMarkerId})`"
          />
          <path
            :d="connection.path"
            class="connection-flow"
            :style="{ '--flow-delay': `${connection.index * -0.27}s` }"
            :stroke="`url(#${lineGradientId})`"
          />
          <circle
            class="signal-dot"
            r="3.25"
            fill="#F0BE97"
            :filter="`url(#${glowFilterId})`"
          >
            <animateMotion
              :path="connection.path"
              :begin="`${connection.index * -0.43}s`"
              :dur="`${3.1 + (connection.index % 3) * 0.25}s`"
              repeatCount="indefinite"
            />
          </circle>
        </g>

        <g
          v-for="connection in downstreamConnections"
          :key="`downstream-${connection.id}`"
          class="connection"
          :class="connectionClasses('downstream', connection.id)"
          data-flow-side="downstream"
          :data-node-id="connection.id"
        >
          <path
            :d="connection.path"
            class="connection-base"
            :marker-end="`url(#${arrowMarkerId})`"
          />
          <path
            :d="connection.path"
            class="connection-flow"
            :style="{ '--flow-delay': `${connection.index * -0.34 - 0.5}s` }"
            :stroke="`url(#${lineGradientId})`"
          />
          <circle
            class="signal-dot"
            r="3.25"
            fill="#F0BE97"
            :filter="`url(#${glowFilterId})`"
          >
            <animateMotion
              :path="connection.path"
              :begin="`${connection.index * -0.51 - 0.9}s`"
              :dur="`${2.8 + (connection.index % 2) * 0.3}s`"
              repeatCount="indefinite"
            />
          </circle>
        </g>
      </g>

      <g
        v-for="(node, index) in upstreamLayout"
        :key="node.id"
        class="flow-node flow-node-enter flow-node-upstream"
        :style="{ '--node-delay': `${120 + index * 80}ms`, '--node-tone': node.tone || '#D4A27F' }"
        tabindex="0"
        focusable="true"
        role="img"
        :aria-label="node.label"
        data-flow-side="upstream"
        :data-node-id="node.id"
        @mouseenter="setActiveNode('upstream', node.id)"
        @mouseleave="clearActiveNode('upstream', node.id)"
        @focus="setActiveNode('upstream', node.id)"
        @blur="clearActiveNode('upstream', node.id)"
      >
        <g
          class="node-visual"
          :class="nodeClasses('upstream', node.id)"
        >
          <rect
            :x="node.x"
            :y="node.y"
            :width="node.width"
            :height="node.height"
            class="node-glow"
          />
          <rect
            :x="node.x"
            :y="node.y"
            :width="node.width"
            :height="node.height"
            class="node-box"
          />
          <rect
            :x="node.x + 10"
            :y="node.y + 12"
            width="24"
            height="24"
            rx="6"
            class="node-icon-frame"
          />
          <image
            v-if="node.icon"
            :href="node.icon"
            :x="node.x + 14"
            :y="node.y + 16"
            width="16"
            height="16"
            preserveAspectRatio="xMidYMid meet"
          />
          <text
            v-else
            :x="node.x + 22"
            :y="node.y + 29"
            text-anchor="middle"
            class="node-glyph"
          >{{ node.glyph || node.label.slice(0, 1) }}</text>
          <text
            :x="node.x + 96"
            :y="node.y + 30"
            text-anchor="middle"
            class="node-label"
          >{{ node.label }}</text>
          <circle
            :cx="node.x + node.width - 13"
            :cy="node.y + 13"
            r="2.5"
            class="node-status"
          />
        </g>
      </g>

      <g
        class="hub-enter"
        :class="{ 'hub-enter--active': activeNode }"
      >
        <g class="hub-float">
          <rect
            x="190"
            y="156"
            width="180"
            height="104"
            fill="#D4A27F"
            opacity="0.2"
            :filter="`url(#${softGlowFilterId})`"
          />
          <rect
            class="hub-pulse"
            x="190"
            y="156"
            width="180"
            height="104"
            fill="#D4A27F"
          />
          <path
            d="M190 174V156H208 M352 156H370V174 M370 242V260H352 M208 260H190V242"
            stroke="#F7F3EA"
            stroke-opacity="0.5"
            stroke-width="1.5"
          />
          <text
            x="280"
            y="207"
            text-anchor="middle"
            fill="#26231F"
            font-family="Georgia, 'Times New Roman', serif"
            font-size="28"
            font-weight="700"
            letter-spacing="-0.8"
          >Niffler</text>
          <text
            x="280"
            y="230"
            text-anchor="middle"
            fill="#26231F"
            font-family="ui-sans-serif, system-ui"
            font-size="10.5"
            font-weight="700"
            letter-spacing="1.6"
          >{{ coreSubtitle }}</text>
        </g>
      </g>

      <g
        v-for="(node, index) in downstreamLayout"
        :key="node.id"
        class="flow-node flow-node-enter flow-node-downstream"
        :style="{ '--node-delay': `${860 + index * 90}ms`, '--node-tone': node.tone || '#D4A27F' }"
        tabindex="0"
        focusable="true"
        role="img"
        :aria-label="node.label"
        data-flow-side="downstream"
        :data-node-id="node.id"
        @mouseenter="setActiveNode('downstream', node.id)"
        @mouseleave="clearActiveNode('downstream', node.id)"
        @focus="setActiveNode('downstream', node.id)"
        @blur="clearActiveNode('downstream', node.id)"
      >
        <g
          class="node-visual"
          :class="nodeClasses('downstream', node.id)"
        >
          <rect
            :x="node.x"
            :y="node.y"
            :width="node.width"
            :height="node.height"
            class="node-glow"
          />
          <rect
            :x="node.x"
            :y="node.y"
            :width="node.width"
            :height="node.height"
            class="node-box"
          />
          <rect
            :x="node.x + 8"
            :y="node.y + 13"
            width="24"
            height="24"
            rx="6"
            class="node-icon-frame"
          />
          <image
            v-if="node.icon"
            :href="node.icon"
            :x="node.x + 12"
            :y="node.y + 17"
            width="16"
            height="16"
            preserveAspectRatio="xMidYMid meet"
          />
          <text
            v-else
            :x="node.x + 20"
            :y="node.y + 30"
            text-anchor="middle"
            class="node-glyph"
          >{{ node.glyph || node.label.slice(0, 1) }}</text>
          <text
            :x="node.x + 78"
            :y="node.y + 31"
            text-anchor="middle"
            class="tool-label"
          >{{ node.label }}</text>
          <circle
            :cx="node.x + node.width - 10"
            :cy="node.y + 11"
            r="2.3"
            class="node-status"
          />
        </g>
      </g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, useId } from 'vue'

interface FlowNode {
  id: string
  label: string
  icon?: string
  glyph?: string
  tone?: string
}

type NodeSide = 'upstream' | 'downstream'

const props = defineProps<{
  upstreamNodes: FlowNode[]
  downstreamNodes: FlowNode[]
  coreSubtitle: string
  accessibleTitle: string
  accessibleDescription: string
}>()

const instanceId = `home-api-${useId().replace(/:/g, '')}`
const titleId = `${instanceId}-title`
const descriptionId = `${instanceId}-description`
const lineGradientId = `${instanceId}-line`
const auraGradientId = `${instanceId}-aura`
const glowFilterId = `${instanceId}-glow`
const softGlowFilterId = `${instanceId}-soft-glow`
const arrowMarkerId = `${instanceId}-arrow`

const upstreamPositions = [
  { x: 12, y: 14, width: 164, height: 48 },
  { x: 198, y: 14, width: 164, height: 48 },
  { x: 384, y: 14, width: 164, height: 48 },
  { x: 12, y: 70, width: 164, height: 48 },
  { x: 198, y: 70, width: 164, height: 48 },
  { x: 384, y: 70, width: 164, height: 48 },
]

const downstreamPositions = [
  { x: 12, y: 310, width: 125, height: 50 },
  { x: 149, y: 310, width: 125, height: 50 },
  { x: 286, y: 310, width: 125, height: 50 },
  { x: 423, y: 310, width: 125, height: 50 },
]

const upstreamPaths = [
  'M94 62 C94 108 238 118 238 149',
  'M280 62 C280 104 280 122 280 149',
  'M466 62 C466 108 322 118 322 149',
  'M94 118 C94 134 238 136 238 149',
  'M280 118 C280 132 280 140 280 149',
  'M466 118 C466 134 322 136 322 149',
]

const downstreamPaths = [
  'M226 267 C226 284 74.5 276 74.5 303',
  'M262 267 C262 282 211.5 284 211.5 303',
  'M298 267 C298 282 348.5 284 348.5 303',
  'M334 267 C334 284 485.5 276 485.5 303',
]

const upstreamLayout = computed(() => props.upstreamNodes.slice(0, upstreamPositions.length).map((node, index) => ({
  ...node,
  ...upstreamPositions[index],
})))

const downstreamLayout = computed(() => props.downstreamNodes.slice(0, downstreamPositions.length).map((node, index) => ({
  ...node,
  ...downstreamPositions[index],
})))

const upstreamConnections = computed(() => upstreamLayout.value.map((node, index) => ({
  id: node.id,
  index,
  path: upstreamPaths[index],
})))

const downstreamConnections = computed(() => downstreamLayout.value.map((node, index) => ({
  id: node.id,
  index,
  path: downstreamPaths[index],
})))

const activeNode = ref<{ side: NodeSide, id: string } | null>(null)

function setActiveNode(side: NodeSide, id: string) {
  activeNode.value = { side, id }
}

function clearActiveNode(side: NodeSide, id: string) {
  if (activeNode.value?.side === side && activeNode.value.id === id) activeNode.value = null
}

function isDimmed(side: NodeSide, id: string) {
  return Boolean(activeNode.value && activeNode.value.side === side && activeNode.value.id !== id)
}

function isRelated(side: NodeSide, id: string) {
  return Boolean(activeNode.value && (activeNode.value.side !== side || activeNode.value.id === id))
}

function nodeClasses(side: NodeSide, id: string) {
  return {
    'node-visual--dimmed': isDimmed(side, id),
    'node-visual--related': isRelated(side, id),
  }
}

function connectionClasses(side: NodeSide, id: string) {
  return {
    'connection--dimmed': isDimmed(side, id),
    'connection--related': isRelated(side, id),
  }
}
</script>

<style scoped>
.api-network-visual {
  width: min(100%, 560px);
  aspect-ratio: 28 / 19;
  margin-inline: auto;
}

.network-aura {
  transform-box: fill-box;
  transform-origin: center;
  animation: aura-pulse 3.8s ease-in-out infinite;
}

.orbit {
  transform-box: view-box;
  transform-origin: 280px 208px;
}

.orbit-outer { animation: orbit-clockwise 14s linear infinite; }
.orbit-inner { animation: orbit-counter 18s linear infinite; }

.connection {
  opacity: 1;
  transition: opacity 220ms ease;
}

.connection-base {
  stroke: #d4a27f;
  stroke-opacity: 0.18;
  stroke-width: 1;
  vector-effect: non-scaling-stroke;
  transition: stroke-opacity 220ms ease, stroke-width 220ms ease;
}

.connection-flow {
  fill: none;
  stroke-width: 1.45;
  stroke-linecap: round;
  stroke-dasharray: 7 12;
  vector-effect: non-scaling-stroke;
  animation: line-flow 1.65s linear infinite;
  animation-delay: var(--flow-delay);
  transition: stroke-width 220ms ease;
}

.signal-dot { opacity: 0.92; }
.connection--dimmed { opacity: 0.14; }
.connection--related .connection-base { stroke-opacity: 0.54; stroke-width: 1.35; }
.connection--related .connection-flow { stroke-width: 2; }

.flow-node {
  outline: none;
}

.flow-node-enter {
  opacity: 0;
  transform-box: fill-box;
  transform-origin: center;
  animation: node-arrive 540ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: var(--node-delay);
}

.node-visual {
  opacity: 1;
  transition: opacity 220ms ease, filter 220ms ease;
}

.node-visual--dimmed { opacity: 0.25; }
.node-visual--related { filter: drop-shadow(0 0 7px color-mix(in srgb, var(--node-tone) 46%, transparent)); }

.node-box {
  fill: #f7f3ea;
  fill-opacity: 0.04;
  stroke: #f7f3ea;
  stroke-opacity: 0.2;
  vector-effect: non-scaling-stroke;
  transition: fill-opacity 220ms ease, stroke-opacity 220ms ease, stroke 220ms ease;
}

.node-glow {
  fill: var(--node-tone);
  opacity: 0;
  filter: blur(12px);
  transition: opacity 220ms ease;
}

.node-icon-frame {
  fill: #f7f3ea;
  fill-opacity: 0.92;
  stroke: var(--node-tone);
  stroke-opacity: 0.4;
  vector-effect: non-scaling-stroke;
}

.node-glyph {
  fill: #26231f;
  font-family: ui-sans-serif, system-ui;
  font-size: 13px;
  font-weight: 800;
}

.node-label,
.tool-label {
  fill: #ded8cc;
  font-family: ui-sans-serif, system-ui;
  font-size: 13px;
  font-weight: 650;
}

.tool-label { font-size: 11.5px; }

.node-status {
  fill: var(--node-tone);
  animation: status-blink 2.4s ease-in-out infinite;
}

.flow-node:hover .node-box,
.flow-node:focus-visible .node-box,
.node-visual--related .node-box {
  fill-opacity: 0.09;
  stroke: var(--node-tone);
  stroke-opacity: 0.72;
}

.flow-node:hover .node-glow,
.flow-node:focus-visible .node-glow,
.node-visual--related .node-glow {
  opacity: 0.13;
}

.hub-enter {
  opacity: 0;
  transform-box: fill-box;
  transform-origin: center;
  animation: hub-arrive 650ms cubic-bezier(0.22, 1, 0.36, 1) 610ms forwards;
  transition: filter 220ms ease;
}

.hub-enter--active { filter: drop-shadow(0 0 10px rgba(240, 190, 151, 0.5)); }

.hub-float {
  transform-box: fill-box;
  transform-origin: center;
  animation: hub-float 4s ease-in-out infinite;
}

.hub-pulse {
  transform-box: fill-box;
  transform-origin: center;
  animation: hub-pulse 3s ease-in-out infinite;
}

@keyframes line-flow { to { stroke-dashoffset: -38; } }
@keyframes orbit-clockwise { to { transform: rotate(360deg); } }
@keyframes orbit-counter { to { transform: rotate(-360deg); } }

@keyframes aura-pulse {
  0%, 100% { opacity: 0.65; transform: scale(0.94); }
  50% { opacity: 1; transform: scale(1.08); }
}

@keyframes hub-float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

@keyframes hub-pulse {
  0%, 100% { opacity: 0.93; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.025); }
}

@keyframes hub-arrive {
  from { opacity: 0; transform: scale(0.86); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes node-arrive {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes status-blink {
  0%, 100% { opacity: 0.38; }
  50% { opacity: 1; }
}

@media (max-width: 639px) {
  .api-network-visual {
    width: calc(100% + 32px);
    max-width: none;
    margin-inline: -16px;
  }

  .node-label { font-size: 16px; }
  .tool-label { font-size: 14px; }
}

@media (prefers-reduced-motion: reduce) {
  .network-aura,
  .orbit,
  .connection-flow,
  .hub-enter,
  .hub-float,
  .hub-pulse,
  .flow-node-enter,
  .node-status {
    animation: none;
  }

  .hub-enter,
  .flow-node-enter { opacity: 1; }
  .signal-dot { display: none; }
}
</style>
