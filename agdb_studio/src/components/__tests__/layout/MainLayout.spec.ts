import { describe, it, expect } from 'vitest'

import { mount } from '@vue/test-utils'
import MainLayout from '../../layout/MainLayout.vue'

describe('MainLayout', () => {
  it('renders properly', () => {
    const wrapper = mount(MainLayout, { props: { msg: 'Hello Vitest' } })
    expect(wrapper.text()).toContain('Main layout')
  })
})
