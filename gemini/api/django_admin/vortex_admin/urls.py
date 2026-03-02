from django.contrib import admin
from django.urls import path
from admin.vortex_core.api import api

urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/", api.urls),
]
