<script lang="ts">
	import SvelteMarkdown from 'svelte-markdown';
	import { marked } from 'marked';
	import MarkdownLink from './MarkdownLink.svelte';

	export let source: string;

	let className = '';

	export { className as class };
</script>

<div class="markdown overflow-x-hidden {className}">
	<SvelteMarkdown
		{source}
		renderers={{
			link: MarkdownLink
		}}
	/>
</div>

<style global lang="postcss">
	.markdown {
		@apply text-slate-100;
	}

	.markdown :global(h1),
	.markdown :global(h2) {
		@apply pt-4 pb-1 mb-3 border-b border-slate-500;
	}

	.markdown :global(h1) {
		@apply text-white font-bold text-3xl;
	}

	.markdown :global(h2) {
		@apply text-slate-100 font-semibold text-2xl;
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
		@apply bg-gray-900 text-slate-300 p-2 my-2 overflow-x-auto;
	}

	.markdown :global(code) {
		@apply bg-gray-900 px-1;
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
		@apply border-l-4 border-gray-600 pl-3 my-3 text-slate-400;
	}
</style>
