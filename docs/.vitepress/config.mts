import {defineConfig} from 'vitepress'

export default defineConfig({
    base: '/',

    themeConfig: {
        search: {
            provider: 'local',
            options: {
                locales: {
                    zh: {
                        translations: {
                            button: {
                                buttonText: '搜索',
                                buttonAriaLabel: '搜索'
                            },
                            modal: {
                                displayDetails: '显示详细列表',
                                resetButtonTitle: '重置搜索',
                                backButtonTitle: '关闭搜索',
                                noResultsText: '没有结果',
                                footer: {
                                    selectText: '选择',
                                    selectKeyAriaLabel: '输入',
                                    navigateText: '导航',
                                    navigateUpKeyAriaLabel: '上箭头',
                                    navigateDownKeyAriaLabel: '下箭头',
                                    closeText: '关闭',
                                    closeKeyAriaLabel: 'Esc'
                                }
                            }
                        }
                    }
                }
            }
        }
    },

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
                    {text: 'Platform', link: '/platform/native'},
                    {text: 'Extensions', link: '/extensions/ambi-macros'}
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
                    ],
                    '/extensions/': [
                        {
                            text: 'Extensions',
                            items: [
                                {text: 'ambi-macros', link: '/extensions/ambi-macros'},
                                {text: 'ambi-memory', link: '/extensions/ambi-memory'},
                                {text: 'ambi-pipelines', link: '/extensions/ambi-pipelines'}
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
                    {text: '平台', link: '/zh/platform/native'},
                    {text: '扩展', link: '/zh/extensions/ambi-macros'}
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
                    ],
                    '/zh/extensions/': [
                        {
                            text: '扩展',
                            items: [
                                {text: 'ambi-macros', link: '/zh/extensions/ambi-macros'},
                                {text: 'ambi-memory', link: '/zh/extensions/ambi-memory'},
                                {text: 'ambi-pipelines', link: '/zh/extensions/ambi-pipelines'}
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
    ignoreDeadLinks: false,

    vite: {
        server: {
            port: 5173,
            open: true
        }
    }
})
