<script lang="ts">
	export let href = '';
	export const raw = false;
	export const title = '';
	export const text = '';
</script>

<a {href} target="_blank">
	<slot />
</a>
