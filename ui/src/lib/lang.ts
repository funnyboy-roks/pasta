import { javascript } from '@codemirror/lang-javascript';
import { cpp } from '@codemirror/lang-cpp';
import { css } from '@codemirror/lang-css';
import { go } from '@codemirror/lang-go';
import { html } from '@codemirror/lang-html';
import { java } from '@codemirror/lang-java';
import { markdown } from '@codemirror/lang-markdown';
import { php } from '@codemirror/lang-php';
import { python } from '@codemirror/lang-python';
import { rust } from '@codemirror/lang-rust';
import { sql } from '@codemirror/lang-sql';
import { wast } from '@codemirror/lang-wast';
import { xml } from '@codemirror/lang-xml';
import { yaml } from '@codemirror/lang-yaml';
import { nix } from '@replit/codemirror-lang-nix';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const supported_langs: Record<string, { name: string; language: () => any }> = {
    plain: { name: 'Plaintext', language: () => null },
    javascript: { name: 'JavaScript', language: javascript },
    typescript: { name: 'TypeScript', language: () => javascript({ typescript: true }) },
    cpp: { name: 'C++', language: cpp },
    css: { name: 'CSS', language: css },
    go: { name: 'Go', language: go },
    html: { name: 'HTML', language: html },
    java: { name: 'Java', language: java },
    markdown: { name: 'Markdown', language: markdown },
    php: { name: 'PHP', language: php },
    python: { name: 'Python', language: python },
    rust: { name: 'Rust', language: rust },
    sql: { name: 'SQL', language: sql },
    wast: { name: 'WAST', language: wast },
    xml: { name: 'XML', language: xml },
    yaml: { name: 'YAML', language: yaml },
    nix: { name: 'NIX', language: nix },
};
