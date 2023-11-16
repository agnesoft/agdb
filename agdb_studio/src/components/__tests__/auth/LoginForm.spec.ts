import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import LoginForm from '../../auth/LoginForm.vue'

describe('LoginForm', () => {
    it('renders properly', () => {
        const wrapper = mount(LoginForm)
        expect(wrapper.find('form').exists()).toBe(true)

        // Check if both fields exists
        expect(wrapper.find('input[type="text"]#username').exists()).toBeTruthy()
        expect(wrapper.find('input[type="password"]#password').exists()).toBeTruthy()
    })
})
