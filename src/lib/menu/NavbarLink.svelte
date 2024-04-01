<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Tooltip } from 'bits-ui';
	import { fly } from 'svelte/transition';
  import { page } from '$app/stores';

	export let to: string;
	export let icon: string;
	export let tooltip: string;
  
  $: active = $page.url.pathname === to;
</script>

<Tooltip.Root openDelay={300}>
	<Tooltip.Trigger class="text-2xl rounded-lg p-2
                          {active ? 'text-green-500 bg-gray-800' : 'text-slate-500 hover:bg-gray-800 hover:text-slate-400'}">
		<a href={to}>
			<Icon {icon} />
		</a>
	</Tooltip.Trigger>
	<Tooltip.Content
		class="rounded-lg bg-gray-800 border border-gray-600 text-slate-300 px-4 py-2 shadow-lg"
		transition={fly}
		transitionConfig={{ duration: 100 }}
		side="right"
	>
    <Tooltip.Arrow class="rounded-[2px] border-l border-t border-gray-600" />
		{tooltip}
	</Tooltip.Content>
</Tooltip.Root>
