<script lang="ts">
	import type { Mod, ModContextItem } from '../../types';
	import { Switch } from 'bits-ui';
	import type { MouseEventHandler } from 'svelte/elements';
	import ModItemContext from './ModItemContext.svelte';
	import ProfileModListItemNoContext from './ProfileModListItemNoContext.svelte';
	import Checkbox from '../ui/Checkbox.svelte';

	type Props = {
		mod: Mod;
		selected: boolean;
		contextItems: ModContextItem[];
		locked: boolean;
		ontoggle?: (newState: boolean) => void;
		onclick?: MouseEventHandler<HTMLDivElement>;
	};

	let { mod, selected, contextItems, locked, ontoggle, onclick }: Props = $props();
</script>

<ModItemContext {mod} {locked} {contextItems}>
	<ProfileModListItemNoContext {mod} {selected} {onclick}>
		{#snippet leading()}
			<Checkbox checked={selected} />
		{/snippet}

		{#snippet trailing()}
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
		{/snippet}
	</ProfileModListItemNoContext>
</ModItemContext>
