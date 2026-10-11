<template>
	<div class="logo-loader">
		<img :src="ShroudEditLogoUrl" alt="" class="logo" />
	</div>
</template>

<script setup lang="ts">
import { ShroudEditLogoUrl } from '@shroudedit/assets'
</script>

<style lang="scss" scoped>
.logo-loader {
	display: flex;
	height: 5rem;
	align-items: center;
	justify-content: center;
}

.logo {
	height: 5rem;
	width: 5rem;
	object-fit: contain;
	animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
	0%,
	100% {
		opacity: 0.65;
		transform: scale(0.94);
	}
	50% {
		opacity: 1;
		transform: scale(1);
	}
}
</style>
