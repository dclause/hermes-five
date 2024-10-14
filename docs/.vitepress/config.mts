import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "HermesFive User Guide",
  description: "An open-source Robot Management Interface with no code",
  base: '/hermes-five/',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    siteTitle: 'HermesFive',
    logo: {
      src: '/icons/robot-love-outline.svg',
      // light: '/logos/logo-light.svg',
      // dark: '/logos/logo-dark.svg'
    },
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Getting started', link: '/getting-started' },
      {
        text: 'v0.1',
        items: [
          { text: 'no other versions yet', link: '...'}
        ]
      }
    ],

    sidebar: [
      {
        text: 'Introduction',
        collapsed: false,
        items: [
          { text: 'What is HermesFive ?', link: '/what-is-HermesFive' },
          { text: 'Getting started', link: '/getting-started' },
          { text: 'Troubleshooting', link: '/troubleshooting' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/dclause/hermes-five' }
    ],
    search: {
      provider: 'local'
    },
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Dominique CLAUSE'
    }
  },
  head: [
    ['link', { rel: "icon", type: "image/png", sizes: "32x32", href: "/logos/favicon-32x32.png"}],
    ['link', { rel: "icon", type: "image/png", sizes: "16x16", href: "/logos/favicon-16x16.png"}],
  ],
  cleanUrls: true,
})
