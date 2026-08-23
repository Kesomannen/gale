<script lang="ts">
	import Markdown from '$lib/components/ui/Markdown.svelte';
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import type { MarkdownType } from '$lib/types';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		open?: boolean;
		content?: string | null;
		type: MarkdownType;
	};

	let { open = $bindable(false), content = null, type }: Props = $props();
</script>

<Dialog large bind:open>
	{#if content}
		<Markdown source={content} />
	{:else}
		<div class="text-primary-600 dark:text-primary-300 flex items-center justify-center gap-2">
			{m.modInfoDialog_noFound({ type })}
		</div>
	{/if}
</Dialog>
