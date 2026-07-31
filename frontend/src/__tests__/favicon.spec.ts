import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

describe('browser theme favicons', () => {
  it('provides black and white icons for light and dark browser themes', () => {
    const indexPath = resolve(process.cwd(), 'index.html')
    const html = readFileSync(indexPath, 'utf8')
    const document = new DOMParser().parseFromString(html, 'text/html')

    const lightIcon = document.querySelector<HTMLLinkElement>(
      'link[rel="icon"][media="(prefers-color-scheme: light)"]',
    )
    const darkIcon = document.querySelector<HTMLLinkElement>(
      'link[rel="icon"][media="(prefers-color-scheme: dark)"]',
    )

    expect(lightIcon?.getAttribute('href')).toBe('%BASE_URL%niffler-logo.svg')
    expect(darkIcon?.getAttribute('href')).toBe('%BASE_URL%niffler-logo-dark.svg')
  })
})
