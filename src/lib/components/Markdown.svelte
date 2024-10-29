<script lang="ts">
	import type { Plugin } from 'svelte-exmarkdown';
	import Markdown, { denylist } from 'svelte-exmarkdown';
	import { gfmPlugin } from 'svelte-exmarkdown/gfm';
	import MarkdownLink from './MarkdownLink.svelte';
	import rehypeRaw from 'rehype-raw';
	import 'highlight.js/styles/atom-one-dark.css';
	import csharp from 'highlight.js/lib/languages/csharp';
	import json from 'highlight.js/lib/languages/json';
	import xml from 'highlight.js/lib/languages/xml';
	import rehypeHighlight from 'rehype-highlight';

	export let source: string;

	let className = '';

	const plugins: Plugin[] = [
		gfmPlugin(),
		denylist(['script', 'iframe', 'object', 'embed', 'base', 'meta', 'link', 'style', 'title']),
		{ rehypePlugin: [rehypeRaw] },
		{
			rehypePlugin: [rehypeHighlight, { languages: { csharp, json, xml }, ignoreMissing: true }]
		},
		{ renderer: { a: MarkdownLink } }
	];

	export { className as class };
</script>

<div class="markdown overflow-x-hidden {className}">
	<Markdown md={source} {plugins} />
</div>

<style global lang="postcss">
	.markdown {
		@apply text-slate-100;
	}

	.markdown :global(h1),
	.markdown :global(h2) {
		@apply mb-3 border-b border-slate-500 pb-1 pt-4;
	}

	.markdown :global(h1) {
		@apply text-3xl font-bold text-white;
	}

	.markdown :global(h2) {
		@apply text-2xl font-semibold text-slate-100;
	}

	.markdown :global(h3) {
		@apply pb-1 pt-2 text-xl font-semibold;
	}

	.markdown :global(h4) {
		@apply pb-0.5 pt-1 text-lg font-medium;
	}

	.markdown :global(a) {
		@apply text-accent-400;
	}

	.markdown :global(a):hover {
		@apply text-accent-300 underline;
	}

	.markdown :global(p) {
		@apply my-3;
	}

	.markdown :global(li) {
		@apply my-1;
	}

	.markdown :global(ul li) {
		@apply ml-8 list-disc pl-0.5;
	}

	.markdown :global(ol li) {
		@apply ml-8 list-decimal pl-0.5;
	}

	.markdown :global(pre) {
		@apply my-2 overflow-x-auto rounded bg-slate-900 p-4 text-slate-300;
	}

	.markdown :global(pre .hljs) {
		@apply p-0;
	}

	.markdown :global(code) {
		@apply bg-slate-900;
	}

	.markdown :global(img) {
		@apply my-2;
	}

	.markdown :global(table) {
		@apply max-w-full border-collapse overflow-x-auto border-2 border-slate-950;
	}

	.markdown :global(th) {
		@apply bg-slate-950 px-3 py-1 text-left font-semibold;
	}

	.markdown :global(tr) {
		@apply border-b border-slate-900;
	}

	.markdown :global(tr:nth-child(2n)) {
		@apply bg-slate-900;
	}

	.markdown :global(td) {
		@apply px-2 py-1 text-left;
	}

	.markdown :global(hr) {
		@apply my-4 border-slate-500;
	}

	.markdown :global(blockquote) {
		@apply my-3 border-l-4 border-slate-600 pl-3 text-slate-400;
	}

	.markdown :global(summary) {
		@apply cursor-pointer;
	}
</style>
