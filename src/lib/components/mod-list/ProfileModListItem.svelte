<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import { Switch } from 'bits-ui';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemContext from './ModItemContext.svelte';
	import { formatModName, isOutdated, modIconSrc } from '$lib/util';
	import Icon from '@iconify/svelte';

	type Props = {
		mod: Mod;
		selected: boolean;
		contextItems: ModContextItem[];
		locked: boolean;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLDivElement>;
	};

	let { mod, selected: selected, contextItems, locked, ontoggle, onclick }: Props = $props();
</script>

<ModItemContext {mod} {locked} {contextItems}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		{onclick}
		role="button"
		tabindex="0"
		class={[
			'group text-primary-300 grid w-full grid-cols-[2fr_1fr_auto] items-center rounded-lg border p-2 lg:grid-cols-[2fr_1fr_1fr_auto]',
			selected ? 'border-primary-500 bg-primary-700' : 'hover:bg-primary-700 border-transparent'
		]}
	>
		<div class="flex items-center overflow-hidden">
			<img src={modIconSrc(mod)} alt={mod.name} class="mr-3 size-10 rounded-sm" />

			<div
				class={[mod.enabled ? 'text-white' : 'line-through', 'mr-2 shrink truncate font-medium']}
			>
				{formatModName(mod.name)}
			</div>

			{#if mod.isPinned}
				<Icon class="text-primary-400 mr-1 shrink-0" icon="mdi:pin" />
			{/if}
			{#if mod.isDeprecated}
				<Icon class="mr-1 shrink-0 text-red-500" icon="mdi:error" />
			{/if}
			{#if isOutdated(mod)}
				<Icon class="text-accent-500 shrink-0" icon="mdi:arrow-up-circle" />
			{/if}
		</div>

		<div class="hidden lg:block">
			{mod.author}
		</div>

		<div>
			{mod.version}
		</div>

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="contents" onclick={(evt) => evt.stopPropagation()}>
			<Switch.Root
				disabled={locked}
				checked={mod.enabled ?? true}
				onCheckedChange={ontoggle}
				class="group data-[state=checked]:bg-accent-700 data-[state=checked]:hover:bg-accent-600 bg-primary-600 hover:bg-primary-500 mr-1 flex h-6 w-12 shrink-0 rounded-full px-1 py-1"
			>
				<Switch.Thumb
					class="data-[state=checked]:bg-accent-200 bg-primary-300 pointer-events-none h-full w-4 rounded-full transition-transform duration-75 ease-out data-[state=checked]:translate-x-6"
				/>
			</Switch.Root>
		</div>
	</div>
</ModItemContext>
