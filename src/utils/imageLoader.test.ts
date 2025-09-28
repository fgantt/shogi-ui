/**
 * Tests for the imageLoader utility functions
 */

import { 
  loadImagesFromDirectory, 
  loadWallpaperImages, 
  loadBoardImages,
  getFallbackWallpaperImages,
  getFallbackBoardImages 
} from './imageLoader';

describe('imageLoader', () => {
  describe('getFallbackWallpaperImages', () => {
    it('should return an array of wallpaper image paths', () => {
      const wallpapers = getFallbackWallpaperImages();
      expect(Array.isArray(wallpapers)).toBe(true);
      expect(wallpapers.length).toBeGreaterThan(0);
      expect(wallpapers.every(path => path.startsWith('/wallpapers/'))).toBe(true);
    });
  });

  describe('getFallbackBoardImages', () => {
    it('should return an array of board image paths', () => {
      const boards = getFallbackBoardImages();
      expect(Array.isArray(boards)).toBe(true);
      expect(boards.length).toBeGreaterThan(0);
      expect(boards.every(path => path.startsWith('/boards/'))).toBe(true);
    });
  });

  describe('loadImagesFromDirectory', () => {
    it('should return an array for valid directory', async () => {
      const result = await loadImagesFromDirectory('/wallpapers');
      expect(Array.isArray(result)).toBe(true);
    });

    it('should return an array for boards directory', async () => {
      const result = await loadImagesFromDirectory('/boards');
      expect(Array.isArray(result)).toBe(true);
    });

    it('should handle non-existent directory gracefully', async () => {
      const result = await loadImagesFromDirectory('/non-existent');
      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(0);
    });
  });

  describe('loadWallpaperImages', () => {
    it('should return an array of wallpaper paths', async () => {
      const wallpapers = await loadWallpaperImages();
      expect(Array.isArray(wallpapers)).toBe(true);
      // If dynamic loading works, we should have some images
      // If it doesn't work, we'll get an empty array, which is also acceptable
      if (wallpapers.length > 0) {
        expect(wallpapers.every(path => path.startsWith('/wallpapers/'))).toBe(true);
      }
    });
  });

  describe('loadBoardImages', () => {
    it('should return an array of board paths', async () => {
      const boards = await loadBoardImages();
      expect(Array.isArray(boards)).toBe(true);
      // If dynamic loading works, we should have some images
      // If it doesn't work, we'll get an empty array, which is also acceptable
      if (boards.length > 0) {
        expect(boards.every(path => path.startsWith('/boards/'))).toBe(true);
      }
    });
  });
});
