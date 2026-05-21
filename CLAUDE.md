# Hermes Desktop - AI Assistant 工作方式

## 核心原则

### 1. 强制使用 iterative-retrieval 模式
在修改任何代码前，必须：
1. **DISPATCH** - 广泛搜索相关文件（至少 3-5 个相关文件）
2. **EVALUATE** - 评估每个文件的相关性（高/中/低）
3. **REFINE** - 学习代码库使用的术语和模式
4. **LOOP** - 最多 3 轮迭代，直到获得足够上下文

**必须找到**：
- ✅ 调用者（谁在用这段代码）
- ✅ 被调用者（这段代码依赖什么）
- ✅ 设计模式（是 Flux？MVC？组件化？）
- ✅ 代码风格（命名、格式、结构）
- ✅ 相似代码（是否可以复用）

### 2. 禁止说"做不了"
遇到复杂任务时：
- ❌ 绝对禁止说"这个任务超出我的能力范围"
- ❌ 绝对禁止在未尝试的情况下放弃
- ✅ 必须先进行 iterative-retrieval 分析
- ✅ 必须基于分析给出明确的边界或方案

### 3. 必须使用 brainstorming 技能
在写代码前，必须：
- 理解核心问题（purpose、constraints、success criteria）
- 探索 2-3 种方案及其权衡
- 明确推荐方案及理由
- 将设计分解为 200-300 字的小节，逐步验证

### 4. 必须使用 writing-plans 技能
将方案分解为具体步骤：
- 每个步骤有明确目标和验证方法
- 按顺序执行，不跳跃
- 每步完成后验证再继续

### 5. 优先使用 test-driven-development
- RED - 先写失败的测试
- GREEN - 写最小代码通过测试
- REFACTOR - 优化代码

### 6. 必须在 verification-before-completion 前完成
完成前必须验证：
- 测试全部通过
- 功能符合需求
- 没有破坏现有功能

## 错误模式警告

如果出现以下迹象，请用户立即干预：
- 🚨 我说"做不了"或表现出困惑
- 🚨 我没有计划就开始修改代码
- 🚨 我只看了 1-2 个文件就开始写代码
- 🚨 我写出的代码与其他代码风格不一致
- 🚨 我跳过了测试步骤

## 项目特定信息

- **项目类型**：Electron + React + TypeScript 桌面应用
- **国际化**：使用 src/shared/i18n 系统，8 种语言支持
- **主要功能**：Hermes Desktop - AI 助手桌面客户端
- **技术栈**：Tauri (Rust backend) + React + TypeScript

## 常用命令

项目根目录：
```
d:\OneManager\hermes-desktop
```

主要目录：
- 前端：`src/renderer/src/`
- 后端：`src-tauri/src/`
- 国际化：`src/shared/i18n/locales/`
- 组件：`src/renderer/src/screens/`

运行和构建：
- 开发：`npm run dev` 或 `npm run tauri dev`
- 构建：`npm run build` 或 `npm run tauri build`
- 测试：`npm run test` 或 `npm run vitest`

---

**最后更新**：2026-05-21
**版本**：1.0
