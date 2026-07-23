import type { Component } from 'vue'
import { BookOpen, CircleDollarSign, HelpCircle, KeyRound, Library } from 'lucide-vue-next'

export interface GuideNavItem {
  id: string
  name: string
  path: string
  icon: Component
  description: string
}

export function createGuideNavItems(t: (key: string) => string): GuideNavItem[] {
  return [
    { id: 'start', name: t('guide.nav.start'), path: '/guide', icon: BookOpen, description: t('guide.nav.startDesc') },
    { id: 'models', name: t('guide.nav.models'), path: '/guide/models', icon: Library, description: t('guide.nav.modelsDesc') },
    { id: 'clients', name: t('guide.nav.clients'), path: '/guide/clients', icon: KeyRound, description: t('guide.nav.clientsDesc') },
    { id: 'billing', name: t('guide.nav.billing'), path: '/guide/usage-billing', icon: CircleDollarSign, description: t('guide.nav.billingDesc') },
    { id: 'faq', name: t('guide.nav.faq'), path: '/guide/faq', icon: HelpCircle, description: t('guide.nav.faqDesc') },
  ]
}
