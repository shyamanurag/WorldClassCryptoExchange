// src/store/store.ts
import { configureStore } from '@reduxjs/toolkit';
import { setupListeners } from '@reduxjs/toolkit/query';

// Import your slices here
// import userReducer from './slices/userSlice';
// import marketReducer from './slices/marketSlice';
// import orderReducer from './slices/orderSlice';

export const store = configureStore({
  reducer: {
    // Add your reducers here
    // user: userReducer,
    // market: marketReducer,
    // order: orderReducer,
  },
  middleware: (getDefaultMiddleware) => 
    getDefaultMiddleware({
      serializableCheck: false,
    }),
});

// Enable listener behavior for the store
setupListeners(store.dispatch);

// Define RootState and AppDispatch types
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;
