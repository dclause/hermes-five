import defineVersionedConfig from "vitepress-versioning-plugin";
import {tabsMarkdownPlugin} from "vitepress-plugin-tabs";

// https://vitepress.dev/reference/site-config
export default defineVersionedConfig({
    title: "Hermes-Five User Guide",
    description: "An open-source Robot Management Interface with no code",
    base: '/hermes-five/',
    cleanUrls: true,
    head: [
        ['link', {rel: "icon", type: "image/png", sizes: "16x16", href: "/hermes-five/favicons/favicon-16x16.png"}],
        ['link', {rel: "icon", type: "image/png", sizes: "32x32", href: "/hermes-five/favicons/favicon-32x32.png"}],
    ],

    markdown: {
        config(md) {
            md.use(tabsMarkdownPlugin)
        }
    },

    themeConfig: {
        // https://vitepress.dev/reference/default-theme-config
        siteTitle: 'Hermes-Five',
        logo: {
            src: '/icons/robot-love-outline.svg',
        },
        versionSwitcher: false,
        nav: [
            {text: 'Home', link: '/'},
            {text: 'Getting started', link: '/guide/introduction/getting-started', activeMatch: '/guide/introduction/'},
            {text: 'Showcases', link: '/guide/showcases', activeMatch: '/guide/showcases'},
            {text: 'API', link: 'https://docs.rs/hermes-five/latest'},
            { component: 'VersionSwitcher' },
        ],
        sidebar: { '/': [
                {
                    text: 'Introduction',
                    items: [
                        {text: 'Getting started', link: '/guide/introduction/getting-started'},
                        {text: 'Concepts & Overview', link: '/guide/introduction/concepts'},
                        {text: 'Troubleshooting', link: '/guide/introduction/troubleshooting'},
                    ]
                },
                {
                    text: 'Showcases',
                    link: '/guide/showcases',
                    items: [
                        {text: 'Protocols & Transports', link: '/guide/showcases/protocols'},
                        {text: 'Board', link: '/guide/showcases/board'},
                        {text: 'Led'},
                        {text: 'Servo'},
                    ]
                }
            ]
        },

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

    versioning: {
        latestVersion: "0.1.0",
        sidebars: {},
    },
}, __dirname);
