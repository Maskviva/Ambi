import {defineConfig} from 'vitepress'

export default defineConfig({
    base: '/',

    locales: {
        root: {
            label: 'English',
            lang: 'en-US',
            title: 'Ambi Docs',
            description: 'A flexible, multi-backend AI agent framework',

            themeConfig: {
                logo: '/logo-text.svg',
                nav: [
                    {text: 'Home', link: '/'},
                    {text: 'Guide', link: '/guide/getting-started'},
                    {text: 'Advanced', link: '/advanced/architecture'},
                    {text: 'Platform', link: '/platform/native'}
                ],

                sidebar: {
                    '/guide/': [
                        {
                            text: 'Guide',
                            items: [
                                {text: 'Getting Started', link: '/guide/getting-started'},
                                {text: 'Basic Agent', link: '/guide/basic-agent'},
                                {text: 'Tools', link: '/guide/tools'},
                                {text: 'Configuration', link: '/guide/configuration'},
                                {text: 'Streaming', link: '/guide/streaming'},
                                {text: 'Multimodal Input', link: '/guide/multimodal'}
                            ]
                        }
                    ],
                    '/advanced/': [
                        {
                            text: 'Advanced Topics',
                            items: [
                                {text: 'Architecture Overview', link: '/advanced/architecture'},
                                {text: 'Custom Engine', link: '/advanced/custom-engine'},
                                {text: 'Custom Pipeline', link: '/advanced/custom-pipeline'},
                                {text: 'Stream Formatter', link: '/advanced/stream-formatter'},
                                {text: 'Tool Parser', link: '/advanced/tool-parser'},
                                {text: 'Context Eviction', link: '/advanced/context-eviction'}
                            ]
                        }
                    ],
                    '/platform/': [
                        {
                            text: 'Platform',
                            items: [
                                {text: 'Native (Linux/Win/Mac)', link: '/platform/native'},
                                {text: 'WebAssembly (WASM)', link: '/platform/wasm'}
                            ]
                        }
                    ]
                },

                socialLinks: [
                    {icon: 'github', link: 'https://github.com/Maskviva/Ambi'}
                ],
                footer: {
                    message: 'Released under the Apache-2.0 License.',
                    copyright: 'Copyright © 2024-present Ambi contributors'
                }
            }
        },

        zh: {
            label: '简体中文',
            lang: 'zh-CN',
            link: '/zh/',
            title: 'Ambi 文档',
            description: '灵活、多后端、可定制的 AI 智能体框架',

            themeConfig: {
                logo: '/logo-text.svg',
                nav: [
                    {text: '首页', link: '/zh/'},
                    {text: '指南', link: '/zh/guide/getting-started'},
                    {text: '高阶开发', link: '/zh/advanced/architecture'},
                    {text: '平台', link: '/zh/platform/native'}
                ],

                sidebar: {
                    '/zh/guide/': [
                        {
                            text: '指南',
                            items: [
                                {text: '快速开始', link: '/zh/guide/getting-started'},
                                {text: '基础 Agent', link: '/zh/guide/basic-agent'},
                                {text: '工具调用', link: '/zh/guide/tools'},
                                {text: '配置详解', link: '/zh/guide/configuration'},
                                {text: '流式响应', link: '/zh/guide/streaming'},
                                {text: '多模态输入', link: '/zh/guide/multimodal'}
                            ]
                        }
                    ],
                    '/zh/advanced/': [
                        {
                            text: '高阶开发',
                            items: [
                                {text: '架构概览', link: '/zh/advanced/architecture'},
                                {text: '自定义引擎', link: '/zh/advanced/custom-engine'},
                                {text: '自定义管道', link: '/zh/advanced/custom-pipeline'},
                                {text: '流式格式化器', link: '/zh/advanced/stream-formatter'},
                                {text: '工具解析器', link: '/zh/advanced/tool-parser'},
                                {text: '上下文驱逐', link: '/zh/advanced/context-eviction'}
                            ]
                        }
                    ],
                    '/zh/platform/': [
                        {
                            text: '平台',
                            items: [
                                {text: '原生平台 (Linux/Win/Mac)', link: '/zh/platform/native'},
                                {text: 'WebAssembly (WASM)', link: '/zh/platform/wasm'}
                            ]
                        }
                    ]
                },

                socialLinks: [
                    {icon: 'github', link: 'https://github.com/Maskviva/Ambi'}
                ],
                footer: {
                    message: '基于 Apache-2.0 协议开源',
                    copyright: '版权所有 © 2024 Ambi 贡献者'
                }
            }
        }
    },

    head: [
        ['link', {rel: 'icon', href: '/logo.svg'}]
    ],
    cleanUrls: true,
    ignoreDeadLinks: false,

    vite: {
        server: {
            port: 5173,
            open: true
        }
    }
})
