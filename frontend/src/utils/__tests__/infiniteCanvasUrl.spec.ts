import { describe, expect, it } from 'vitest'

import { getInfiniteCanvasUrl } from '../infiniteCanvasUrl'

describe('getInfiniteCanvasUrl', () => {
  it('builds the canvas URL for a root deployment', () => {
    expect(getInfiniteCanvasUrl('canvas', '/')).toBe('/InfiniteCanvas/canvas')
  })

  it('preserves the Vite base path for a GitHub Pages deployment', () => {
    expect(getInfiniteCanvasUrl('canvas', '/Niffler/')).toBe('/Niffler/InfiniteCanvas/canvas')
  })

  it('normalizes base and child path slashes', () => {
    expect(getInfiniteCanvasUrl('/canvas', '/Niffler')).toBe('/Niffler/InfiniteCanvas/canvas')
    expect(getInfiniteCanvasUrl('', '/Niffler///')).toBe('/Niffler/InfiniteCanvas/')
  })
})
