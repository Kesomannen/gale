<script lang="ts">
	import { invokeCommand } from '$lib/invoke';
	import type { LaunchMode, PrefValue } from '$lib/models';
	import Icon from '@iconify/svelte';
	import { Select, type Selected } from 'bits-ui';
	import { onMount } from 'svelte';

	import { slide } from 'svelte/transition';

	let open = false;

	let value: LaunchMode | undefined;
	let instances = 1;

	onMount(async () => {
		value = (await invokeCommand<PrefValue>('get_pref', { key: 'launch_mode' })) as LaunchMode;
		instances = value.content?.instances ?? 1;
	});

	function set(selection: Selected<string> | undefined) {
		if (selection === undefined) return;
		if (value === undefined) return;

		value.type = selection.value as 'steam' | 'direct';

		if (value.type == 'direct') {
			value.content = { instances };
		} else {
			value.content = undefined;
		}

		invokeCommand('set_pref', { key: 'launch_mode', value });
	}
</script>

<div class="flex items-center">
	<div class="text-slate-300 min-w-60 truncate">Launch mode</div>

	<Select.Root
		items={[
			{ value: 'steam', label: 'Steam' },
			{ value: 'direct', label: 'Direct' }
		]}
		selected={value === undefined
			? undefined
			: { value: value.type, label: value.type.charAt(0).toUpperCase() + value.type.slice(1) }}
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
			<Select.Item
				value="steam"
				class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
			>
				<Icon icon="mdi:steam" class="text-lg mr-1" />
				Steam
				<Select.ItemIndicator class="ml-auto">
					<Icon icon="mdi:check" class="text-green-400 text-lg" />
				</Select.ItemIndicator>
			</Select.Item>
			<Select.Item
				value="direct"
				class="flex items-center px-3 py-1 truncate text-slate-400 hover:text-slate-200 text-left rounded-md hover:bg-gray-700 cursor-default"
			>
				<Icon icon="mdi:launch" class="text-lg mr-1" />
				Direct
				<Select.ItemIndicator class="ml-auto">
					<Icon icon="mdi:check" class="text-green-400 text-lg" />
				</Select.ItemIndicator>
			</Select.Item>
		</Select.Content>
	</Select.Root>
	<input
		type="number"
		step="int32"
		disabled={value?.type !== 'direct'}
		bind:value={instances}
		on:input={() => set({ value: 'direct', label: 'Direct' })}
		class="px-3 py-1 rounded-lg bg-gray-900 ml-1
	 			text-slate-300 hover:text-slate-100 disabled:text-slate-400 border border-gray-500 border-opacity-0 enabled:hover:border-opacity-100"
	/>
</div>
