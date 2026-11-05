#include <stdio.h>
int custom_add_assign(int,int);int default_add_assign(int,int);
int custom_sub_assign(int,int);int default_sub_assign(int,int);
int custom_mul_assign(int,int);int default_mul_assign(int,int);
int custom_div_assign(int,int);int default_div_assign(int,int);
int custom_rem_assign(int,int);int default_rem_assign(int,int);
int custom_bitxor_assign(int,int);int default_bitxor_assign(int,int);
int custom_bitand_assign(int,int);int default_bitand_assign(int,int);
int custom_bitor_assign(int,int);int default_bitor_assign(int,int);
int custom_shl_assign(int,int);int default_shl_assign(int,int);
int custom_shr_assign(int,int);int default_shr_assign(int,int);
int main(void){volatile int right=23;printf("%d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d %d\n",custom_add_assign(7,right),custom_sub_assign(7,right),custom_mul_assign(7,right),custom_div_assign(7,right),custom_rem_assign(7,right),custom_bitxor_assign(7,right),custom_bitand_assign(7,right),custom_bitor_assign(7,right),custom_shl_assign(7,right),custom_shr_assign(7,right),default_add_assign(7,right),default_sub_assign(7,right),default_mul_assign(7,right),default_div_assign(7,right),default_rem_assign(7,right),default_bitxor_assign(7,right),default_bitand_assign(7,right),default_bitor_assign(7,right),default_shl_assign(7,right),default_shr_assign(7,right));return 0;}
