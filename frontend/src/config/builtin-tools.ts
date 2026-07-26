import { Mail, Shield, AlertTriangle } from 'lucide-vue-next'
import type { LucideIcon } from 'lucide-vue-next'

type Translate = (key: string) => string

export interface BuiltinTool {
  name: string
  description: string
  href: string
  icon: LucideIcon
}

const BUILTIN_TOOL_DEFINITIONS = [
  {
    nameKey: 'staticUi.emailConfig',
    descriptionKey: 'staticUi.emailConfigDesc',
    href: '/admin/email',
    icon: Mail,
  },
  {
    nameKey: 'staticUi.ipSecurity',
    descriptionKey: 'staticUi.ipSecurityDesc',
    href: '/admin/ip-security',
    icon: Shield,
  },
  {
    nameKey: 'staticUi.auditLogs',
    descriptionKey: 'staticUi.auditLogsDesc',
    href: '/admin/audit-logs',
    icon: AlertTriangle,
  },
] as const

export function createBuiltinTools(translate: Translate): BuiltinTool[] {
  return BUILTIN_TOOL_DEFINITIONS.map(tool => ({
    name: translate(tool.nameKey),
    description: translate(tool.descriptionKey),
    href: tool.href,
    icon: tool.icon,
  }))
}

/** href → display name mapping for breadcrumbs */
export function createBuiltinToolBreadcrumbs(translate: Translate): Record<string, string> {
  return Object.fromEntries(createBuiltinTools(translate).map(tool => [tool.href, tool.name]))
}
