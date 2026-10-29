import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/controller/:id',
      name: 'controller-detail',
      component: () => import('../views/ControllerDetailView.vue'),
      props: true,
    },
    {
      path: '/controller/new',
      name: 'controller-new',
      component: () => import('../views/ControllerFormView.vue'),
    },
    {
      path: '/controller/:id/edit',
      name: 'controller-edit',
      component: () => import('../views/ControllerFormView.vue'),
      props: true,
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/',
    },
  ],
})

export default router
