/**
 * Audio utility for playing sound effects in the shogi game
 */

class AudioManager {
  private static instance: AudioManager;
  private audioContext: AudioContext | null = null;
  private pieceMoveSound: HTMLAudioElement | null = null;
  private checkmateSound: HTMLAudioElement | null = null;
  private drawSound: HTMLAudioElement | null = null;
  private soundsEnabled: boolean = true;
  private volume: number = 0.9; // Default volume (0.0 to 1.0)

  private constructor() {
    // Load volume from localStorage if available
    const storedVolume = localStorage.getItem('shogi-sound-volume');
    if (storedVolume) {
      this.volume = parseFloat(storedVolume);
    }
    
    this.initializeAudio();
  }

  public static getInstance(): AudioManager {
    if (!AudioManager.instance) {
      AudioManager.instance = new AudioManager();
    }
    return AudioManager.instance;
  }

  private async initializeAudio(): Promise<void> {
    try {
      // Create audio context
      this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
      
      // Try to load piece move sound, fallback to synthetic sound if not available
      try {
        this.pieceMoveSound = new Audio('/sounds/piece-move.mp3');
        this.pieceMoveSound.preload = 'auto';
        this.pieceMoveSound.volume = this.volume;
        
        // Test if the file exists
        this.pieceMoveSound.addEventListener('error', () => {
          console.log('Piece move sound file not found, will use synthetic sound');
          this.pieceMoveSound = null;
        });
      } catch (error) {
        console.log('Using synthetic piece move sound');
        this.pieceMoveSound = null;
      }
      
      // Try to load game over sounds
      try {
        this.checkmateSound = new Audio('/sounds/checkmate.mp3');
        this.checkmateSound.preload = 'auto';
        this.checkmateSound.volume = this.volume * 0.9;
        this.checkmateSound.addEventListener('error', () => {
          console.log('Checkmate sound file not found, will use synthetic sound');
          this.checkmateSound = null;
        });
      } catch (error) {
        console.log('Using synthetic checkmate sound');
        this.checkmateSound = null;
      }
      
      try {
        this.drawSound = new Audio('/sounds/draw.mp3');
        this.drawSound.preload = 'auto';
        this.drawSound.volume = this.volume * 0.8;
        this.drawSound.addEventListener('error', () => {
          console.log('Draw sound file not found, will use synthetic sound');
          this.drawSound = null;
        });
      } catch (error) {
        console.log('Using synthetic draw sound');
        this.drawSound = null;
      }
      
      // Handle audio context suspension (required for user interaction)
      if (this.audioContext.state === 'suspended') {
        // We'll resume it when the first sound is played
      }
    } catch (error) {
      console.warn('Failed to initialize audio:', error);
    }
  }

  public setSoundsEnabled(enabled: boolean): void {
    console.log('[AudioManager] Setting sounds enabled:', enabled);
    this.soundsEnabled = enabled;
  }

  public isSoundsEnabled(): boolean {
    return this.soundsEnabled;
  }

  public setVolume(volume: number): void {
    // Clamp volume between 0 and 1
    this.volume = Math.max(0, Math.min(1, volume));
    
    // Update volume on all loaded audio elements
    if (this.pieceMoveSound) {
      this.pieceMoveSound.volume = this.volume;
    }
    if (this.checkmateSound) {
      this.checkmateSound.volume = this.volume * 0.9; // Slightly quieter
    }
    if (this.drawSound) {
      this.drawSound.volume = this.volume * 0.8; // Even quieter
    }
  }

  public getVolume(): number {
    return this.volume;
  }

  public async playPreviewSound(): Promise<void> {
    // Play preview sound regardless of soundsEnabled setting (for testing volume)
    try {
      // Resume audio context if suspended
      if (this.audioContext && this.audioContext.state === 'suspended') {
        await this.audioContext.resume();
      }

      // Use audio file if available, otherwise generate synthetic sound
      if (this.pieceMoveSound) {
        // Clone the audio element to allow concurrent playback without conflicts
        const soundClone = this.pieceMoveSound.cloneNode(true) as HTMLAudioElement;
        soundClone.volume = this.volume;
        
        // Play the cloned sound
        try {
          await soundClone.play();
        } catch (error) {
          console.warn('Preview sound play failed:', error);
        }
      } else if (this.audioContext) {
        await this.playSyntheticClackSound();
      }
    } catch (error) {
      console.warn('Failed to play preview sound:', error);
    }
  }

  public async playPieceMoveSound(): Promise<void> {
    if (!this.soundsEnabled) {
      console.log('[AudioManager] Sounds disabled, skipping piece move sound');
      return;
    }

    try {
      // Resume audio context if suspended (required for user interaction)
      if (this.audioContext && this.audioContext.state === 'suspended') {
        console.log('[AudioManager] Resuming suspended audio context');
        await this.audioContext.resume();
      }

      // Use audio file if available, otherwise generate synthetic sound
      if (this.pieceMoveSound) {
        console.log('[AudioManager] Playing piece move sound from file, volume:', this.volume);
        
        // Clone the audio element to allow concurrent playback without conflicts
        const soundClone = this.pieceMoveSound.cloneNode(true) as HTMLAudioElement;
        soundClone.volume = this.volume;
        
        // Play the cloned sound
        try {
          await soundClone.play();
        } catch (error) {
          console.warn('Piece move sound play failed:', error);
        }
      } else if (this.audioContext) {
        console.log('[AudioManager] Playing synthetic clack sound, volume:', this.volume);
        await this.playSyntheticClackSound();
      } else {
        console.warn('[AudioManager] No audio context or sound file available');
      }
    } catch (error) {
      console.warn('Failed to play piece move sound:', error);
    }
  }

  private async playSyntheticClackSound(): Promise<void> {
    if (!this.audioContext) return;

    const sampleRate = this.audioContext.sampleRate;
    const duration = 0.15; // 150ms
    const bufferSize = Math.floor(sampleRate * duration);
    const buffer = this.audioContext.createBuffer(1, bufferSize, sampleRate);
    const data = buffer.getChannelData(0);

    // Generate a sharp clack sound using noise and envelope
    for (let i = 0; i < bufferSize; i++) {
      const t = i / sampleRate;
      // Sharp attack with quick decay
      const envelope = Math.exp(-t * 25) * (1 - t / duration);
      // White noise with some filtering
      const noise = (Math.random() * 2 - 1) * 0.3;
      // Add a slight click at the beginning
      const click = t < 0.01 ? Math.sin(t * 2000 * Math.PI) * 0.5 : 0;
      
      data[i] = (noise + click) * envelope;
    }

    const source = this.audioContext.createBufferSource();
    const gainNode = this.audioContext.createGain();
    
    source.buffer = buffer;
    gainNode.gain.value = 0.6 * this.volume; // Apply user volume setting
    
    source.connect(gainNode);
    gainNode.connect(this.audioContext.destination);
    
    source.start();
  }

  public async playCustomSound(soundPath: string, volume: number = 0.7): Promise<void> {
    if (!this.soundsEnabled) {
      return;
    }

    try {
      const audio = new Audio(soundPath);
      audio.volume = volume;
      audio.currentTime = 0;
      await audio.play();
    } catch (error) {
      console.warn('Failed to play custom sound:', error);
    }
  }

  public async playCheckmateSound(): Promise<void> {
    if (!this.soundsEnabled) {
      return;
    }

    try {
      // Resume audio context if suspended
      if (this.audioContext && this.audioContext.state === 'suspended') {
        await this.audioContext.resume();
      }

      // Use audio file if available, otherwise generate synthetic sound
      if (this.checkmateSound) {
        // Clone the audio element to allow concurrent playback without conflicts
        const soundClone = this.checkmateSound.cloneNode(true) as HTMLAudioElement;
        soundClone.volume = this.volume * 0.9;
        
        // Play the cloned sound
        try {
          await soundClone.play();
        } catch (error) {
          console.warn('Checkmate sound play failed:', error);
        }
      } else if (this.audioContext) {
        await this.playSyntheticVictorySound();
      }
    } catch (error) {
      console.warn('Failed to play checkmate sound:', error);
    }
  }

  public async playDrawSound(): Promise<void> {
    if (!this.soundsEnabled) {
      return;
    }

    try {
      // Resume audio context if suspended
      if (this.audioContext && this.audioContext.state === 'suspended') {
        await this.audioContext.resume();
      }

      // Use audio file if available, otherwise generate synthetic sound
      if (this.drawSound) {
        // Clone the audio element to allow concurrent playback without conflicts
        const soundClone = this.drawSound.cloneNode(true) as HTMLAudioElement;
        soundClone.volume = this.volume * 0.8;
        
        // Play the cloned sound
        try {
          await soundClone.play();
        } catch (error) {
          console.warn('Draw sound play failed:', error);
        }
      } else if (this.audioContext) {
        await this.playSyntheticDrawSound();
      }
    } catch (error) {
      console.warn('Failed to play draw sound:', error);
    }
  }

  private async playSyntheticVictorySound(): Promise<void> {
    if (!this.audioContext) return;

    const sampleRate = this.audioContext.sampleRate;
    const duration = 0.8; // 800ms
    const bufferSize = Math.floor(sampleRate * duration);
    const buffer = this.audioContext.createBuffer(1, bufferSize, sampleRate);
    const data = buffer.getChannelData(0);

    // Generate a triumphant ascending tone sequence
    const frequencies = [440, 554, 659, 880]; // A, C#, E, A (major triad)
    const noteDuration = duration / frequencies.length;
    
    for (let i = 0; i < bufferSize; i++) {
      const t = i / sampleRate;
      const noteIndex = Math.floor(t / noteDuration);
      if (noteIndex >= frequencies.length) break;
      
      const freq = frequencies[noteIndex];
      const noteTime = t - (noteIndex * noteDuration);
      const envelope = Math.exp(-noteTime * 3) * (1 - noteTime / noteDuration);
      
      data[i] = Math.sin(2 * Math.PI * freq * noteTime) * envelope * 0.3;
    }

    const source = this.audioContext.createBufferSource();
    const gainNode = this.audioContext.createGain();
    
    source.buffer = buffer;
    gainNode.gain.value = 0.5 * this.volume; // Apply user volume setting
    
    source.connect(gainNode);
    gainNode.connect(this.audioContext.destination);
    
    source.start();
  }

  private async playSyntheticDrawSound(): Promise<void> {
    if (!this.audioContext) return;

    const sampleRate = this.audioContext.sampleRate;
    const duration = 0.5; // 500ms
    const bufferSize = Math.floor(sampleRate * duration);
    const buffer = this.audioContext.createBuffer(1, bufferSize, sampleRate);
    const data = buffer.getChannelData(0);

    // Generate a neutral, settling tone
    const frequencies = [523, 440]; // C to A (neutral sound)
    const noteDuration = duration / frequencies.length;
    
    for (let i = 0; i < bufferSize; i++) {
      const t = i / sampleRate;
      const noteIndex = Math.floor(t / noteDuration);
      if (noteIndex >= frequencies.length) break;
      
      const freq = frequencies[noteIndex];
      const noteTime = t - (noteIndex * noteDuration);
      const envelope = Math.exp(-noteTime * 4) * (1 - noteTime / noteDuration);
      
      data[i] = Math.sin(2 * Math.PI * freq * noteTime) * envelope * 0.25;
    }

    const source = this.audioContext.createBufferSource();
    const gainNode = this.audioContext.createGain();
    
    source.buffer = buffer;
    gainNode.gain.value = 0.4 * this.volume; // Apply user volume setting
    
    source.connect(gainNode);
    gainNode.connect(this.audioContext.destination);
    
    source.start();
  }
}

// Export singleton instance
export const audioManager = AudioManager.getInstance();

// Convenience functions
export const playPieceMoveSound = () => audioManager.playPieceMoveSound();
export const playCheckmateSound = () => audioManager.playCheckmateSound();
export const playDrawSound = () => audioManager.playDrawSound();
export const playPreviewSound = () => audioManager.playPreviewSound();
export const setSoundsEnabled = (enabled: boolean) => audioManager.setSoundsEnabled(enabled);
export const isSoundsEnabled = () => audioManager.isSoundsEnabled();
export const setVolume = (volume: number) => audioManager.setVolume(volume);
export const getVolume = () => audioManager.getVolume();
