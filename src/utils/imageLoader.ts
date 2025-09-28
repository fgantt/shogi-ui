/**
 * Utility functions for dynamically loading wallpaper and board images
 */

/**
 * Dynamically loads image paths from the public directory
 * This function creates a dynamic import that scans for image files
 * @param directory - The directory path relative to public (e.g., '/wallpapers' or '/boards')
 * @param extensions - Array of file extensions to include (e.g., ['jpg', 'png', 'svg', 'jpeg', 'webp'])
 * @returns Promise<string[]> - Array of image paths
 */
export async function loadImagesFromDirectory(
  directory: string,
  extensions: string[] = ['jpg', 'png', 'svg', 'jpeg', 'webp']
): Promise<string[]> {
  try {
    // Create a glob pattern for the directory
    const globPattern = `${directory}/**/*.{${extensions.join(',')}}`;
    
    // Use import.meta.glob to dynamically import all matching files
    const modules = import.meta.glob('/public/**/*.{jpg,png,svg,jpeg,webp}', { eager: true });
    
    // Filter modules that match our directory and extensions
    const imagePaths: string[] = [];
    
    for (const [path, module] of Object.entries(modules)) {
      // Convert the import path to a public URL path
      // e.g., '/public/wallpapers/image.jpg' -> '/wallpapers/image.jpg'
      const publicPath = path.replace('/public', '');
      
      // Check if this path is in our target directory
      if (publicPath.startsWith(directory)) {
        imagePaths.push(publicPath);
      }
    }
    
    return imagePaths.sort();
  } catch (error) {
    console.error(`Error loading images from ${directory}:`, error);
    return [];
  }
}

/**
 * Loads all wallpaper images from the /wallpapers directory
 * @returns Promise<string[]> - Array of wallpaper image paths
 */
export async function loadWallpaperImages(): Promise<string[]> {
  return loadImagesFromDirectory('/wallpapers', ['jpg', 'png', 'svg', 'jpeg']);
}

/**
 * Loads all board images from the /boards directory
 * @returns Promise<string[]> - Array of board image paths
 */
export async function loadBoardImages(): Promise<string[]> {
  return loadImagesFromDirectory('/boards', ['jpg', 'png', 'svg', 'jpeg', 'webp']);
}

/**
 * Fallback function that returns hardcoded image lists if dynamic loading fails
 * This ensures the app still works even if import.meta.glob is not available
 */
export function getFallbackWallpaperImages(): string[] {
  return [
    '/wallpapers/beautiful-japanese-garden.jpg',
    '/wallpapers/beautiful-natural-landscape.jpg',
    '/wallpapers/fuji1.jpg',
    '/wallpapers/koi.jpg',
    '/wallpapers/maple.jpg',
    '/wallpapers/mountain-house.jpeg',
    '/wallpapers/photo1.jpg',
    '/wallpapers/shogi-background-placeholder.svg',
    '/wallpapers/wave.jpg',
    '/wallpapers/woman-with-kimono-wagasa-umbrella.jpg'
  ];
}

export function getFallbackBoardImages(): string[] {
  return [
    '/boards/koi-bw.jpg',
    '/boards/doubutsu.png',
    '/boards/marble-calacatta.jpg',
    '/boards/marble.jpg',
    '/boards/quartz-1.jpg',
    '/boards/quartz-2.jpg',
    '/boards/stars-1.jpg',
    '/boards/stars-2.jpg',
    '/boards/wood-agathis-1.jpg',
    '/boards/wood-agathis-2.jpg',
    '/boards/wood-bambo.jpg',
    '/boards/wood-boxwood-1.jpg',
    '/boards/wood-boxwood-2.jpg',
    '/boards/wood-boxwood-3.jpg',
    '/boards/wood-boxwood-4.jpg',
    '/boards/wood-cherry-1.jpg',
    '/boards/wood-cherry-2.jpg',
    '/boards/wood-cherry-3.jpg',
    '/boards/wood-cypress-1.jpg',
    '/boards/wood-ginkgo-1.jpg',
    '/boards/wood-ginkgo-2.jpg',
    '/boards/wood-ginkgo-3.jpg',
    '/boards/wood-hiba-1.jpeg',
    '/boards/wood-hickory-1.jpg',
    '/boards/wood-katsura-1.png',
    '/boards/wood-mahogany-1.jpg',
    '/boards/wood-maple-1.jpg',
    '/boards/wood-maple-2.webp',
    '/boards/wood-pecan-1.jpg',
    '/boards/wood-pecan-2.jpg',
    '/boards/wood-red-spruce-1.jpg'
  ];
}
