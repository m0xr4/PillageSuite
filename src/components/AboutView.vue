<template>
  <div class="about-container">
    <h1 class="about-title">About Pillage Suite</h1>
    
    <section class="about-section">
      <h2>About the Tool</h2>
      <p>
        Pillage Suite is a powerful graphical interface for exploring Network Shares in an Active Directory environment.
        It provides tools for the indexing and querying of files, folders, shares as well as identities from Active Directory. 
        Once indexed, the tool allows Security Professionals and System Administrators to query the database to uncover potential security issues.
      </p>
    </section>
    
    <section class="about-section">
      <h2>Author</h2>
      <p>
        Developed and maintained by Moritz Rauch, contributions are welcome :) <br><br>
        <a href="#" @click.prevent="openInBrowser('https://github.com/m0xr4')">https://github.com/m0xr4</a>
      </p>
    </section>

    <section class="about-section">
      <h2>License</h2>
      
        <h3>GNU General Public License (GPL3)</h3>
        <p>
          This application is licensed under the <a href="#" @click.prevent="openInBrowser('https://www.gnu.org/licenses/gpl-3.0.en.html')">GNU General Public License v3.0</a>.
        </p>
        <p>
          The GPLv3 is a strong copyleft license that guarantees this software and all its derivatives will always remain free and open source. 
          Anyone can use, study, share, and modify the code, but any changes or redistributions must also be licensed under GPLv3, ensuring the software and its improvements stay open for everyone.
        </p>
        <ul>
          <li>You are free to use, share, and modify this software for any purpose.</li>
          <li>Any modifications or derivative works must also be released under the GPLv3 license.</li>
          <li>The source code must always be made available when distributing the software or any modified versions.</li>
          <li>You must include the original copyright and license notice in all copies or substantial portions of the software.</li>
          <li>All changes and modifications must be documented and remain open source.</li>
        </ul>
      
    </section>
    
    <section class="about-section">
      <h2>Version</h2>
      <p>1.0</p>
    </section>
  </div>
</template>

<script setup>
import { openUrl } from '@tauri-apps/plugin-opener';

const openInBrowser = async (url) => {
  try {
    await openUrl(url);
  } catch (error) {
    console.error('Failed to open URL:', error);
  }
};
</script>

<style scoped>
.about-container {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  color: var(--color-text);
  background-color: var(--color-background);
  border-radius: 0.5rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.about-title {
  font-size: 1.8rem;
  color: var(--color-primary);
  margin-bottom: 1.5rem;
  text-align: center;
  border-bottom: 1px solid var(--color-border);
  padding-bottom: 1rem;
}

.about-section {
  margin-bottom: 2rem;
}

.about-section h2 {
  font-size: 1.5rem;
  color: var(--color-heading);
  margin-bottom: 0.75rem;
  border-left: 3px solid var(--color-primary);
  padding-left: 0.75rem;
}

.about-section h3 {
  font-size: 1.2rem;
  margin-bottom: 0.5rem;
}

.about-section p {
  line-height: 1.6;
  margin-bottom: 1rem;
}

.about-section a {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: 500;
  cursor: pointer;
}

.about-section a:hover {
  text-decoration: underline;
}

.license-container {
  background-color: var(--color-background-soft);
  padding: 1rem;
  border-radius: 0.375rem;
  border-left: 3px solid var(--color-primary);
}

.license-container ul {
  margin: 1rem 0;
  padding-left: 1.5rem;
}

.license-container ul li {
  margin-bottom: 0.5rem;
  line-height: 1.4;
}

.license-container a {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: 500;
}

.license-container a:hover {
  text-decoration: underline;
}
</style> 