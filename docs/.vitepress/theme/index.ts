// .vitepress/theme/index.ts
import DefaultTheme from 'vitepress/theme'
import VersionSwitcher from 'vitepress-versioning-plugin/src/components/VersionSwitcher.vue'
import { enhanceAppWithTabs } from 'vitepress-plugin-tabs/client'

// Import style fixes and customizations.
import './custom.css'
import { Theme } from 'vitepress';

export default {
    extends: DefaultTheme,
    enhanceApp({ app }) {
        app.component('VersionSwitcher', VersionSwitcher);
        enhanceAppWithTabs(app);
    }
} satisfies Theme;
