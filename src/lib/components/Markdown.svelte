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

	type Props = {
		source: string;
		class?: string;
	};

	let { source, class: className = '' }: Props = $props();

	const plugins: Plugin[] = [
		gfmPlugin(),
		denylist(['script', 'iframe', 'object', 'embed', 'base', 'meta', 'link', 'style', 'title']),
		{ rehypePlugin: [rehypeRaw] },
		{
			rehypePlugin: [rehypeHighlight, { languages: { csharp, json, xml }, ignoreMissing: true }]
		},
		{ renderer: { a: MarkdownLink } }
	];
</script>

<div class="markdown overflow-x-hidden {className}">
	<Markdown md={source} {plugins} />
</div>
