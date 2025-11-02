<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { ClassValue } from 'clsx';
	import Info from '../ui/Info.svelte';
	import profiles from '$lib/state/profile.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		class?: ClassValue;
	};

	let { class: classProp }: Props = $props();
</script>

<div
	class={[
		classProp,
		'bg-primary-700 text-primary-300 flex items-center rounded-lg py-1.5 pr-1 pl-3'
	]}
>
	<Icon icon="mdi:lock" class="mr-2 text-xl" />
	<span class="mr-auto">{m.profileLockedBanner_title()}</span>

	<Info>{m.profileLockedBanner_content({name : profiles.active?.sync?.owner.displayName ?? m.unknown()})}</Info>
</div>
