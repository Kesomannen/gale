<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import { Switch } from 'bits-ui';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemWithContext from './ModItemWithContext.svelte';

	type Props = {
		mod: Mod;
		isSelected: boolean;
		contextItems: ModContextItem[];
		locked: boolean;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLButtonElement>;
	};

	let { mod, isSelected, contextItems, locked, ontoggle, onclick }: Props = $props();
</script>

<ModItemWithContext {mod} {isSelected} {onclick} {locked} {contextItems}>
	<!-- make sure click events don't propagate and cause the mod to be selected -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
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
</ModItemWithContext>
