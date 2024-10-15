import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

function App() {
  const [micData, setMicData] = useState<number[]>([]);

  useEffect(() => {
    const unlisten = listen('mic-data', event => {
      console.log('Received mic data:', event.payload);  // デバッグメッセージを追加
      setMicData(event.payload as number[]);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  return (
    <div>
      <h1>マイクからのデータ</h1>
      <pre>{JSON.stringify(micData, null, 2)}</pre>
    </div>
  );
}

export default App;