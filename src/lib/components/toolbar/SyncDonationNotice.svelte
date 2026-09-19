<script lang="ts">
	import Link from '../ui/Link.svelte';
	import Icon from '@iconify/svelte';
	import InfoBox from '../ui/InfoBox.svelte';
	import { m } from '$lib/paraglide/messages';

	type Props = {
		show?: boolean;
		closedAt: string | null;
	};

	let { show: showProp = true, closedAt = $bindable() }: Props = $props();

	const closeDuration = 1000 * 60 * 60 * 24 * 7; // 1 week

	let show = $derived(
		showProp && (!closedAt || Date.now() - new Date(closedAt).getTime() > closeDuration)
	);
</script>

<InfoBox class={!show && 'hidden'}>
	<div class="text-primary-900 text-lg font-semibold dark:text-white">
		{m.syncDonationNotice_content_1()}
	</div>

	<div class="text-primary-600 dark:text-primary-300">
		{m.syncDonationNotice_content_2()}<Link href="https://ko-fi.com/kesomannen">Kofi</Link>

		<Icon class="mb-1 inline" icon="mdi:heart" />.
	</div>

	<button
		class="text-primary-500 hover:text-accent-600 dark:text-primary-400 dark:hover:text-accent-400 mt-2 flex items-center gap-1 text-sm hover:underline"
		onclick={() => {
			closedAt = new Date().toISOString();
		}}
	>
		<Icon icon="mdi:close" />
		{m.syncDonationNotice_button()}
	</button>
</InfoBox>
