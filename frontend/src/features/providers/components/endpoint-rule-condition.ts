import type { BodyRuleCondition, BodyRuleConditionLeaf, BodyRuleConditionOp } from '@/api/endpoints'

export type ConditionSource = 'body' | 'original' | 'request_headers'
export type ConditionGroupMode = 'all' | 'any'

export interface EditableConditionLeaf {
  kind: 'leaf'
  path: string
  op: BodyRuleConditionOp
  value: string
  source: ConditionSource
}

export interface EditableConditionGroup {
  kind: 'group'
  mode: ConditionGroupMode
  children: EditableConditionNode[]
}

export type EditableConditionNode = EditableConditionLeaf | EditableConditionGroup

export const CONDITION_OP_OPTIONS: Array<{ value: BodyRuleConditionOp; label: string }> = [
  { value: 'eq', label: i18n.global.t('endpointRuleUi.eq') },
  { value: 'neq', label: i18n.global.t('endpointRuleUi.neq') },
  { value: 'gt', label: i18n.global.t('endpointRuleUi.gt') },
  { value: 'lt', label: i18n.global.t('endpointRuleUi.lt') },
  { value: 'gte', label: i18n.global.t('endpointRuleUi.gte') },
  { value: 'lte', label: i18n.global.t('endpointRuleUi.lte') },
  { value: 'starts_with', label: i18n.global.t('endpointRuleUi.startsWith') },
  { value: 'ends_with', label: i18n.global.t('endpointRuleUi.endsWith') },
  { value: 'contains', label: i18n.global.t('endpointRuleUi.contains') },
  { value: 'matches', label: i18n.global.t('endpointRuleUi.matches') },
  { value: 'exists', label: i18n.global.t('endpointRuleUi.exists') },
  { value: 'not_exists', label: i18n.global.t('endpointRuleUi.notExists') },
  { value: 'in', label: i18n.global.t('endpointRuleUi.in') },
  { value: 'type_is', label: i18n.global.t('endpointRuleUi.typeIs') },
]

const NUMERIC_OPS = new Set(['gt', 'lt', 'gte', 'lte'])
const STRING_OPS = new Set(['starts_with', 'ends_with'])
const TYPE_IS_VALUES = new Set(['string', 'number', 'boolean', 'array', 'object', 'null'])

export function createEmptyConditionLeaf(): EditableConditionLeaf {
  return {
    kind: 'leaf',
    path: '',
    op: 'eq',
    value: '',
    source: 'body',
  }
}

export function createConditionGroup(
  mode: ConditionGroupMode = 'all',
  children: EditableConditionNode[] = [createEmptyConditionLeaf()],
): EditableConditionGroup {
  return {
    kind: 'group',
    mode,
    children,
  }
}

export function cloneEditableCondition(node: EditableConditionNode): EditableConditionNode {
  if (node.kind === 'group') {
    return {
      kind: 'group',
      mode: node.mode,
      children: node.children.map(cloneEditableCondition),
    }
  }
  return { ...node }
}

export function conditionToEditable(condition?: BodyRuleCondition | null): EditableConditionNode | null {
  if (!condition) return null
  if ('all' in condition && Array.isArray(condition.all)) {
    return createConditionGroup(
      'all',
      condition.all.map(child => conditionToEditable(child) || createEmptyConditionLeaf()),
    )
  }
  if ('any' in condition && Array.isArray(condition.any)) {
    return createConditionGroup(
      'any',
      condition.any.map(child => conditionToEditable(child) || createEmptyConditionLeaf()),
    )
  }
  if (!isBodyRuleConditionLeaf(condition)) {
    return null
  }
  const source = (condition as { source?: unknown }).source
  return {
    kind: 'leaf',
    path: condition.path || '',
    op: condition.op || 'eq',
    value: condition.value !== undefined
      ? (typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value))
      : '',
    source: source === 'request_headers' || source === 'headers'
        ? 'request_headers'
        : source === 'original'
          ? 'original'
      : 'body',
  }
}

function isBodyRuleConditionLeaf(condition: BodyRuleCondition): condition is BodyRuleConditionLeaf {
  return 'path' in condition && 'op' in condition
}

export function editableConditionToApi(node: EditableConditionNode | null): BodyRuleCondition | undefined {
  if (!node) return undefined

  if (node.kind === 'group') {
    const children = node.children
      .map(child => editableConditionToApi(child))
      .filter((child): child is BodyRuleCondition => !!child)
    if (!children.length) return undefined
    return node.mode === 'all' ? { all: children } : { any: children }
  }

  const path = node.path.trim()
  if (!path) return undefined

  const base = {
    path,
    op: node.op,
    ...(node.source === 'request_headers'
      ? { source: 'request_headers' as const }
      : node.source === 'original'
        ? { source: 'original' as const }
        : {}),
  }

  if (node.op === 'exists' || node.op === 'not_exists') {
    return base
  }

  const raw = node.value.trim()
  if (!raw) {
    return { ...base, value: '' }
  }

  try {
    return { ...base, value: JSON.parse(raw) }
  } catch {
    return { ...base, value: raw }
  }
}

export function isConditionValueRequired(op: BodyRuleConditionOp): boolean {
  return op !== 'exists' && op !== 'not_exists'
}

export function getConditionValuePlaceholder(op: BodyRuleConditionOp): string {
  if (op === 'in') return '["a", "b"]'
  if (op === 'type_is') return 'string/number/boolean/...'
  return i18n.global.t('endpointRuleUi.value')
}

export function getBodyRuleConditionPathPlaceholder(path: string): string {
  return path.includes('[*]') || /\[\d+-\d+\]/.test(path) ? `$item.${i18n.global.t('endpointRuleUi.fieldName')}` : i18n.global.t('endpointRuleUi.fieldPath')
}

export function conditionEquals(
  left: EditableConditionNode | null,
  right: EditableConditionNode | null,
): boolean {
  if (left === right) return true
  if (!left || !right) return false
  if (left.kind !== right.kind) return false

  if (left.kind === 'group' && right.kind === 'group') {
    if (left.mode !== right.mode) return false
    if (left.children.length !== right.children.length) return false
    return left.children.every((child, i) => conditionEquals(child, right.children[i]))
  }

  if (left.kind === 'leaf' && right.kind === 'leaf') {
    return left.path === right.path
      && left.op === right.op
      && left.value === right.value
      && left.source === right.source
  }

  return false
}

export function validateEditableCondition(node: EditableConditionNode | null): string | null {
  if (!node) return null

  if (node.kind === 'group') {
    if (!node.children.length) return i18n.global.t('endpointRuleUi.childRequired')
    for (let i = 0; i < node.children.length; i += 1) {
      const err = validateEditableCondition(node.children[i])
      if (err) return i18n.global.t('endpointRuleUi.childError', { index: i + 1, error: err })
    }
    return null
  }

  const path = node.path.trim()
  if (!path) return i18n.global.t('endpointRuleUi.pathRequired')

  if (!isConditionValueRequired(node.op)) return null

  const raw = node.value.trim()
  let parsed: unknown = raw
  if (raw) {
    try {
      parsed = JSON.parse(raw)
    } catch {
      parsed = raw
    }
  }

  if (NUMERIC_OPS.has(node.op)) {
    if (typeof parsed !== 'number' || Number.isNaN(parsed)) return i18n.global.t('endpointRuleUi.numberRequired')
    return null
  }

  if (STRING_OPS.has(node.op)) {
    if (typeof parsed !== 'string') return i18n.global.t('endpointRuleUi.stringRequired')
    return null
  }

  if (node.op === 'matches') {
    if (typeof parsed !== 'string' || !parsed) return i18n.global.t('endpointRuleUi.regexRequired')
    try {
      new RegExp(parsed)
      return null
    } catch (error: unknown) {
      return i18n.global.t('endpointRuleUi.invalidRegex', { error: error instanceof Error ? error.message : String(error) })
    }
  }

  if (node.op === 'in') {
    if (!Array.isArray(parsed)) return i18n.global.t('endpointRuleUi.arrayRequired')
    return null
  }

  if (node.op === 'type_is') {
    if (typeof parsed !== 'string' || !TYPE_IS_VALUES.has(parsed)) {
      return i18n.global.t('endpointRuleUi.typeIsAllowed')
    }
  }

  return null
}
import { i18n } from '@/i18n'
