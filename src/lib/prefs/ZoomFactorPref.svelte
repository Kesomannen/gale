<script lang="ts">
	import Label from '$lib/components/Label.svelte';
import { invokeCommand } from '$lib/invoke';
	import type { LaunchMode, PrefValue, SelectItem } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Select, type Selected } from 'bits-ui';
	import { onMount } from 'svelte';

	import { slide } from 'svelte/transition';

	const items: SelectItem[] = [
		{ label: '50%', value: '0.5' },
		{ label: '75%', value: '0.75' },
		{ label: '100%', value: '1' },
		{ label: '125%', value: '1.25' },
		{ label: '150%', value: '1.5' },
	];

	let open = false;

	let value: number;
	let selectedItem: SelectItem;

	onMount(async () => {
		value = (await invokeCommand<PrefValue>('get_pref', { key: 'zoom_factor' })) as number;
		selectedItem = items.find((item) => item.value === value.toString())!;
	});

	function set(selection: Selected<string> | undefined) {
		if (selection === undefined) return;
		if (value === undefined) return;

		value = Number.parseFloat(selection.value);
		invokeCommand('set_pref', { key: 'zoom_factor', value });
	}
</script>

<div class="flex items-center">
	<Label text="Zoom factor">
		Changes the zoom level of the mod manager.
	</Label>

	<Select.Root
		items={items}
		selected={selectedItem}
		onSelectedChange={set}
	>
		<Select.Trigger
			class="flex items-center flex-grow bg-gray-900 rounded-lg px-3 py-1
							border border-gray-500 border-opacity-0 hover:border-opacity-100"
		>
			<Select.Value class="text-slate-300 text-left w-full" />
			<Icon
				class="text-slate-400 text-xl ml-auto transition-all
												transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
				icon="mdi:chevron-down"
			/>
		</Select.Trigger>
		<Select.Content
			class="flex flex-col bg-gray-800 gap-0.5 shadow-xl p-1 rounded-lg border border-gray-600"
			transition={slide}
			transitionConfig={{ duration: 100 }}
		>
			{#each items as item}
				<Select.Item
					value={item.value}
					class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
				>
					{item.label}
					<Select.ItemIndicator class="ml-auto">
						<Icon icon="mdi:check" class="text-green-400 text-lg" />
					</Select.ItemIndicator>
				</Select.Item>
			{/each}
		</Select.Content>
	</Select.Root>
</div>
