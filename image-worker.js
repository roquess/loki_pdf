self.onmessage = async ({ data }) => {
  const { images, quality, scaleFactor, workerId } = data;
  const result = [];

  for (let i = 0; i < images.length; i++) {
    const img = images[i];
    try {
      const blob = new Blob([new Uint8Array(img.data)], { type: img.type || 'image/jpeg' });
      const bitmap = await createImageBitmap(blob);
      
      const w = Math.max(100, Math.floor(bitmap.width * scaleFactor));
      const h = Math.max(100, Math.floor(bitmap.height * scaleFactor));
      
      const canvas = new OffscreenCanvas(w, h);
      const ctx = canvas.getContext('2d');
      ctx.imageSmoothingQuality = 'high';
      ctx.drawImage(bitmap, 0, 0, w, h);
      
      const jpegBlob = await canvas.convertToBlob({ type: 'image/jpeg', quality: quality / 100 });
      const array = new Uint8Array(await jpegBlob.arrayBuffer());
      
      const gain = 1 - array.length / img.data.length;
      if (gain < 0.05) {
        result.push({ ...img, compressed_size: img.data.length });
        continue;
      }

      result.push({
        object_id: img.object_id,
        data: Array.from(array),
        width: w,
        height: h,
        original_size: img.data.length,
        compressed_size: array.length,
        format: 'jpeg',
        gain_pct: Math.round(gain * 100)
      });

      self.postMessage({
        type: 'progress',
        workerId,
        current: i + 1,
        total: images.length,
        saved: img.data.length - array.length
      });

    } catch (err) {
      result.push({ ...img, compressed_size: img.data.length, error: err.message });
    }
  }

  self.postMessage({
    type: 'complete',
    workerId,
    compressed: result,
    totalSaved: result.reduce((s, x) => s + (x.original_size - x.compressed_size), 0)
  });
};

