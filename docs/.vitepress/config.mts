import {defineConfig} from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
    title: "Hermes-Five User Guide",
    description: "An open-source Robot Management Interface with no code",
    base: '/hermes-five/',
    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        siteTitle: 'Hermes-Five',
        logo: {
            src: '/icons/robot-love-outline.svg',
        },
        nav: [
            {text: 'Home', link: '/'},
            {text: 'Getting started', link: '/getting-started'},
            {text: 'API & Examples', link: '/api'},
            {
                text: 'v0.1',
                items: [
                    {text: 'no other versions yet', link: '...'}
                ]
            }
        ],

        sidebar: [
            {
                text: 'Introduction',
                collapsed: false,
                items: [
                    {text: 'What is Hermes-Five ?', link: '/what-is-Hermes-Five'},
                    {text: 'Getting started', link: '/getting-started'},
                    {text: 'Concepts & Overview', link: '/concepts'},
                    {text: 'Troubleshooting', link: '/troubleshooting'},
                ]
            },
            {
                text: 'API & Examples',
                collapsed: false,
                items: [
                    {text: 'Board'},
                    {text: 'Led'},
                    {text: 'Servo'},
                ]
            }
        ],

        socialLinks: [
            {icon: 'github', link: 'https://github.com/dclause/hermes-five'}
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
        ['link', {rel: "icon", type: "image/png", sizes: "16x16", href: "/hermes-five/favicons/favicon-16x16.png"}],
        ['link', {rel: "icon", type: "image/png", sizes: "32x32", href: "/hermes-five/favicons/favicon-32x32.png"}],
    ],
    cleanUrls: true,
})
