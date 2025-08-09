import React from 'react';
import './App.css';

function App() {
  return (
    <div className="App">
      <header className="bg-blue-600 text-white p-4">
        <h1 className="text-2xl font-bold">GeekTools Plugin Marketplace</h1>
        <p className="text-blue-100">插件市场</p>
      </header>
      <main className="container mx-auto px-4 py-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold text-gray-900 mb-4">
            Welcome to the Plugin Marketplace
          </h2>
          <p className="text-gray-600 mb-8">
            Discover and share plugins for GeekTools
          </p>
          <div className="bg-gray-100 rounded-lg p-8">
            <p className="text-gray-500">
              React frontend is now running! 🎉
            </p>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;