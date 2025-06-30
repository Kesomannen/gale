<script lang="ts">
	import { Menubar } from 'bits-ui';
	import type { Snippet } from 'svelte';

	type Props = {
		label: string;
		children?: Snippet;
	};

	let { label, children }: Props = $props();
</script>

<Menubar.Menu>
	<Menubar.Trigger
		class="text-primary-400 hover:bg-primary-700 hover:text-primary-200 mx-[-1px] h-full cursor-default rounded-sm px-3 text-sm"
	>
		{label}
	</Menubar.Trigger>
	<Menubar.Content
		class="border-primary-600 bg-primary-800 mt-0.5 flex flex-col gap-0.5 rounded-lg border py-1 shadow-xl"
	>
		{@render children?.()}
	</Menubar.Content>
</Menubar.Menu>
