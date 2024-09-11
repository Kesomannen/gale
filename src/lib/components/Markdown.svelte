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
		{ rehypePlugin: [rehypeHighlight, { languages: { csharp, json, xml }, ignoreMissing: true }] },
		{ renderer: { a: MarkdownLink } }
	];

	export { className as class };
</script>

<div class="markdown overflow-x-hidden {className}">
	<Markdown md={source} {plugins} />
</div>

<style global lang="postcss">
	.markdown {
		@apply text-gray-100;
	}

	.markdown :global(h1),
	.markdown :global(h2) {
		@apply pt-4 pb-1 mb-3 border-b border-gray-500;
	}

	.markdown :global(h1) {
		@apply text-white font-bold text-3xl;
	}

	.markdown :global(h2) {
		@apply text-gray-100 font-semibold text-2xl;
	}

	.markdown :global(h3) {
		@apply font-semibold text-xl pb-1 pt-2;
	}

	.markdown :global(h4) {
		@apply font-medium text-lg pb-0.5 pt-1;
	}

	.markdown :global(a) {
		@apply text-green-400;
	}

	.markdown :global(a):hover {
		@apply text-green-300 underline;
	}

	.markdown :global(p) {
		@apply my-3;
	}

	.markdown :global(li) {
		@apply my-1;
	}

	.markdown :global(ul li) {
		@apply list-disc ml-8 pl-0.5;
	}

	.markdown :global(ol li) {
		@apply list-decimal ml-8 pl-0.5;
	}

	.markdown :global(pre) {
		@apply bg-gray-900 p-4 text-gray-300 my-2 overflow-x-auto;
	}

	.markdown :global(pre .hljs) {
		@apply p-0;
	}

	.markdown :global(code) {
		@apply bg-gray-900;
	}

	.markdown :global(img) {
		@apply my-2;
	}

	.markdown :global(table) {
		@apply border-2 border-gray-950 border-collapse overflow-x-auto max-w-full;
	}

	.markdown :global(th) {
		@apply bg-gray-950 font-semibold px-3 py-1 text-left;
	}

	.markdown :global(tr) {
		@apply border-b border-gray-900;
	}

	.markdown :global(tr:nth-child(2n)) {
		@apply bg-gray-900;
	}

	.markdown :global(td) {
		@apply px-2 py-1 text-left;
	}

	.markdown :global(hr) {
		@apply border-gray-500 my-4;
	}

	.markdown :global(blockquote) {
		@apply border-l-4 border-gray-600 pl-3 my-3 text-gray-400;
	}

	.markdown :global(summary) {
		@apply cursor-pointer;
	}
</style>
