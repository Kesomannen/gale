<script lang="ts">
	import Icon from '@iconify/svelte';
	import { m } from '$lib/paraglide/messages';

	const KIBIBYTE = 1024;

	type Props = {
		uploaded: number;
		bytes: number;
		removed: number;
		unchanged: number;
	};

	let { uploaded, bytes, removed, unchanged }: Props = $props();

	function formatBytes(bytes: number) {
		if (bytes < KIBIBYTE) return `${bytes} B`;
		const units = ['KB', 'MB', 'GB', 'TB'];
		let value = bytes / KIBIBYTE;
		let unit = 0;
		while (value >= KIBIBYTE && unit < units.length - 1) {
			value /= KIBIBYTE;
			unit++;
		}
		const decimals = value >= 10 ? 1 : 2;
		return `${value.toFixed(decimals)} ${units[unit]}`;
	}
</script>

<div class="mt-5 grid grid-cols-3 gap-3">
	{#each [{ icon: 'mdi:cloud-upload', label: m.deploymentStats_upload(), value: uploaded, detail: formatBytes(bytes) }, { icon: 'mdi:trash-can-outline', label: m.deploymentStats_remove(), value: removed, detail: m.deploymentStats_serverFiles() }, { icon: 'mdi:check-circle-outline', label: m.deploymentStats_unchanged(), value: unchanged, detail: m.deploymentStats_noTransfer() }] as stat}
		<div class="bg-primary-100 dark:bg-primary-900 rounded-lg p-3">
			<div class="text-primary-500 dark:text-primary-400 flex items-center gap-2 text-sm">
				<Icon icon={stat.icon} />
				{stat.label}
			</div>
			<div class="text-primary-800 dark:text-primary-200 mt-1 text-xl font-semibold">
				{stat.value}
			</div>
			<div class="text-primary-500 text-sm">{stat.detail}</div>
		</div>
	{/each}
</div>
